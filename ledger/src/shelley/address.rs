use std::fmt::Write as _;
use std::{fmt::Display, iter};

use bech32::{Bech32, ByteIterExt, Fe32IterExt, Hrp};
use displaydoc::Display;
use thiserror::Error;
use tinycbor::{
    CborLen, Decode, Decoder, Encode, Write,
    container::{self, bounded},
};

use crate::crypto::Blake2b224Digest;
use crate::shelley::{
    Credential, Network,
    credential::{self, Delegation},
};

const HASH_SIZE: usize = 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address<'a> {
    pub payment: Credential<'a>,
    pub stake: Option<Delegation<'a>>,
    pub network: Network,
}

impl<'a> Address<'a> {
    fn header(&self) -> u8 {
        match (self.payment, self.stake) {
            (Credential::VerificationKey(_), Some(Delegation::StakeKey(_))) => 0b0000,
            (Credential::Script(_), Some(Delegation::StakeKey(_))) => 0b0001,
            (Credential::VerificationKey(_), Some(Delegation::Script(_))) => 0b0010,
            (Credential::Script(_), Some(Delegation::Script(_))) => 0b0011,
            (Credential::VerificationKey(_), Some(Delegation::Pointer(_))) => 0b0100,
            (Credential::Script(_), Some(Delegation::Pointer(_))) => 0b0101,
            (Credential::VerificationKey(_), None) => 0b0110,
            (Credential::Script(_), None) => 0b0111,
        }
    }
}

impl Display for Address<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hrp = Hrp::parse_unchecked(match self.network {
            Network::Main => "addr",
            Network::Test => "addr_test",
        });

        let network_magic = self.network as u8;
        let first_byte = (self.header() << 4) | network_magic;

        let iter = iter::once(first_byte).chain(self.payment.as_ref().iter().copied());

        match self.stake {
            Some(Delegation::Script(hash) | Delegation::StakeKey(hash)) => iter
                .chain(*hash)
                .bytes_to_fes()
                .with_checksum::<Bech32>(&hrp)
                .chars()
                .try_for_each(|c| f.write_char(c)),
            Some(Delegation::Pointer(pointer)) => iter
                .chain(pointer)
                .bytes_to_fes()
                .with_checksum::<Bech32>(&hrp)
                .chars()
                .try_for_each(|c| f.write_char(c)),
            None => iter
                .bytes_to_fes()
                .with_checksum::<Bech32>(&hrp)
                .chars()
                .try_for_each(|c| f.write_char(c)),
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for Address<'a> {
    type Error = bounded::Error<InvalidType>;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Address::from_bytes::<true>(value)
    }
}

impl<'a> Address<'a> {
    /// Same as `TryFrom<&[u8]>`, but with an option to allow surplus bytes.
    ///
    /// This is needed for [`crate::address::truncating`].
    pub(crate) fn from_bytes<const STRICT: bool>(
        mut bytes: &'a [u8],
    ) -> Result<Self, bounded::Error<InvalidType>> {
        let first_byte = bytes.first().ok_or(bounded::Error::Missing)?;
        bytes = &bytes[1..];
        let header = first_byte >> 4;
        let network_magic = first_byte & 0b0000_1111;
        let network = match network_magic {
            1 => Network::Main,
            // We default to Test for unknown ids...
            _ => Network::Test,
        };

        let first_hash: &Blake2b224Digest = bytes
            .get(..HASH_SIZE)
            .ok_or(bounded::Error::Missing)?
            .try_into()
            .expect("slice has correct length");
        bytes = &bytes[HASH_SIZE..];

        if header < 0b0100 {
            let second_hash: &Blake2b224Digest = bytes
                .get(..HASH_SIZE)
                .ok_or(bounded::Error::Missing)?
                .try_into()
                .expect("slice has correct length");
            if STRICT && bytes.len() > HASH_SIZE {
                return Err(bounded::Error::Surplus);
            }

            let (payment, stake) = match header {
                0b0000 => (
                    Credential::VerificationKey(first_hash),
                    Delegation::StakeKey(second_hash),
                ),
                0b0001 => (
                    Credential::Script(first_hash),
                    Delegation::StakeKey(second_hash),
                ),
                0b0010 => (
                    Credential::VerificationKey(first_hash),
                    Delegation::Script(second_hash),
                ),
                0b0011 => (
                    Credential::Script(first_hash),
                    Delegation::Script(second_hash),
                ),
                _ => unreachable!(),
            };
            Ok(Address {
                payment,
                stake: Some(stake),
                network,
            })
        } else if header < 0b0110 {
            let mut iter = bytes.iter().copied();
            let pointer = credential::ChainPointer::from_bytes(iter.by_ref())
                .ok_or(bounded::Error::Missing)?;
            if iter.next().is_some() {
                return Err(bounded::Error::Surplus);
            }

            let payment = match header {
                0b0100 => Credential::VerificationKey(first_hash),
                0b0101 => Credential::Script(first_hash),
                _ => unreachable!(),
            };

            Ok(Address {
                payment,
                stake: Some(Delegation::Pointer(pointer)),
                network,
            })
        } else if header < 0b1000 {
            if STRICT && !bytes.is_empty() {
                return Err(bounded::Error::Surplus);
            }

            let payment = match header {
                0b0110 => Credential::VerificationKey(first_hash),
                0b0111 => Credential::Script(first_hash),
                _ => unreachable!(),
            };

            Ok(Address {
                payment,
                stake: None,
                network,
            })
        } else {
            Err(bounded::Error::Content(InvalidType))
        }
    }
}

impl CborLen for Address<'_> {
    fn cbor_len(&self) -> usize {
        let mut len = 1 + HASH_SIZE; // first byte + payment credential

        match &self.stake {
            Some(Delegation::StakeKey(_) | Delegation::Script(_)) => {
                len += HASH_SIZE; // stake credential
            }
            Some(Delegation::Pointer(pointer)) => {
                len += pointer.into_iter().count();
            }
            _ => {}
        }
        len.cbor_len() + len
    }
}

impl Encode for Address<'_> {
    fn encode<W: Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        // `24 < cbor_len < 256` because pointer encoding can't exceed 30 bytes.
        e.0.write_all(&[0x58, self.cbor_len() as u8])?;

        let network_magic = self.network as u8;
        let first_byte = (self.header() << 4) | network_magic;
        e.0.write_all(&[first_byte])?;
        e.0.write_all(self.payment.as_ref())?;

        match self.stake {
            Some(Delegation::StakeKey(hash) | Delegation::Script(hash)) => {
                e.0.write_all(hash)?;
            }
            Some(Delegation::Pointer(pointer)) => {
                for b in pointer {
                    e.0.write_all(&[b])?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl<'a, 'b: 'a> Decode<'b> for Address<'a> {
    type Error = container::Error<bounded::Error<InvalidType>>;

    fn decode(d: &mut Decoder<'b>) -> Result<Self, Self::Error> {
        let data: &[u8] = Decode::decode(d)?;
        Address::try_from(data).map_err(container::Error::Content)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Account<'a> {
    pub credential: Credential<'a>,
    pub network: Network,
}

impl Account<'_> {
    fn header(&self) -> u8 {
        let header = match self.credential {
            Credential::VerificationKey(_) => 0b1110,
            Credential::Script(_) => 0b1111,
        };
        let network_magic = self.network as u8;
        (header << 4) | (network_magic & 0b1111)
    }
}

impl Display for Account<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hrp = Hrp::parse_unchecked(match self.network {
            Network::Main => "stake",
            Network::Test => "stake_test",
        });

        iter::once(self.header())
            .chain(self.credential.as_ref().iter().copied())
            .bytes_to_fes()
            .with_checksum::<Bech32>(&hrp)
            .chars()
            .try_for_each(|c| f.write_char(c))
    }
}

impl<'a> TryFrom<&'a [u8]> for Account<'a> {
    type Error = bounded::Error<super::address::InvalidType>;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        let first_byte = bytes.first().ok_or(bounded::Error::Missing)?;
        let header = first_byte >> 4;
        let network_magic = first_byte & 0b0000_1111;
        let network = match network_magic {
            1 => Network::Main,
            _ => Network::Test,
        };

        let hash: &Blake2b224Digest = bytes
            .get(1..29)
            .ok_or(bounded::Error::Missing)?
            .try_into()
            .expect("slice has correct length");
        if bytes.len() > 29 {
            return Err(bounded::Error::Surplus);
        }

        let credential = if header == 0b1110 {
            Credential::VerificationKey(hash)
        } else if header == 0b1111 {
            Credential::Script(hash)
        } else {
            return Err(bounded::Error::Content(super::address::InvalidType));
        };

        Ok(Account {
            credential,
            network,
        })
    }
}

impl<'a, 'b: 'a> Decode<'b> for Account<'a> {
    type Error = container::Error<bounded::Error<super::address::InvalidType>>;

    fn decode(d: &mut Decoder<'b>) -> Result<Self, Self::Error> {
        let data: &[u8] = Decode::decode(d)?;
        Account::try_from(data).map_err(container::Error::Content)
    }
}

const LEN: usize = 29;

impl CborLen for Account<'_> {
    fn cbor_len(&self) -> usize {
        LEN.cbor_len() + LEN
    }
}

impl Encode for Account<'_> {
    fn encode<W: Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        // CBOR bytestring length 29 header.
        e.0.write_all(&[0x40, LEN as u8])?;
        e.0.write_all(&[self.header()])?;
        e.0.write_all(self.credential.as_ref())
    }
}

/// invalid address type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error, Display)]
pub struct InvalidType;

#[cfg(test)]
mod tests {
    //! All tests are coming from CIP 19
    use super::*;

    const VK: &Blake2b224Digest = &[
        148, 147, 49, 92, 217, 46, 181, 216, 196, 48, 78, 103, 183, 225, 106, 227, 109, 97, 211,
        69, 2, 105, 70, 87, 129, 26, 44, 142,
    ];
    const STAKE_VK: &Blake2b224Digest = &[
        51, 123, 98, 207, 255, 100, 3, 160, 106, 58, 203, 195, 79, 140, 70, 0, 60, 105, 254, 121,
        163, 98, 140, 239, 169, 196, 114, 81,
    ];
    const SCRIPT_HASH: &Blake2b224Digest = &[
        195, 123, 27, 93, 192, 102, 159, 29, 60, 97, 166, 253, 219, 46, 143, 222, 150, 190, 135,
        184, 129, 198, 11, 206, 142, 141, 84, 47,
    ];
    const POINTER: credential::ChainPointer = credential::ChainPointer {
        slot: 2498243,
        tx_index: 27,
        cert_index: 3,
    };

    fn from_bech32<T: TryFrom<&'static [u8], Error: std::fmt::Debug>>(addr: &str) -> T {
        let bytes: &'static _ = bech32::decode(addr).unwrap().1.leak();
        T::try_from(bytes).unwrap()
    }

    #[test]
    fn type0() {
        const ADDR_MAIN: &str = "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x";
        const ADDR_TEST: &str = "addr_test1qz2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgs68faae";

        let main = from_bech32(ADDR_MAIN);
        assert!(matches!(
            main,
            Address {
                payment: Credential::VerificationKey(VK),
                stake: Some(credential::Delegation::StakeKey(STAKE_VK)),
                network: Network::Main
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = from_bech32(ADDR_TEST);
        assert!(matches!(
            test,
            Address {
                payment: Credential::VerificationKey(VK),
                stake: Some(credential::Delegation::StakeKey(STAKE_VK)),
                network: Network::Test
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type1() {
        const ADDR_MAIN: &str = "addr1z8phkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gten0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgs9yc0hh";
        const ADDR_TEST: &str = "addr_test1zrphkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gten0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgsxj90mg";

        let main = from_bech32(ADDR_MAIN);
        assert!(matches!(
            main,
            Address {
                payment: Credential::Script(SCRIPT_HASH),
                stake: Some(credential::Delegation::StakeKey(STAKE_VK)),
                network: Network::Main
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = from_bech32(ADDR_TEST);
        assert!(matches!(
            test,
            Address {
                payment: Credential::Script(SCRIPT_HASH),
                stake: Some(credential::Delegation::StakeKey(STAKE_VK)),
                network: Network::Test
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type2() {
        const ADDR_MAIN: &str = "addr1yx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzerkr0vd4msrxnuwnccdxlhdjar77j6lg0wypcc9uar5d2shs2z78ve";
        const ADDR_TEST: &str = "addr_test1yz2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzerkr0vd4msrxnuwnccdxlhdjar77j6lg0wypcc9uar5d2shsf5r8qx";

        let main = from_bech32(ADDR_MAIN);
        assert!(matches!(
            main,
            Address {
                payment: Credential::VerificationKey(VK),
                stake: Some(credential::Delegation::Script(SCRIPT_HASH)),
                network: Network::Main
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = from_bech32(ADDR_TEST);
        assert!(matches!(
            test,
            Address {
                payment: Credential::VerificationKey(VK),
                stake: Some(credential::Delegation::Script(SCRIPT_HASH)),
                network: Network::Test
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type3() {
        const ADDR_MAIN: &str = "addr1x8phkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gt7r0vd4msrxnuwnccdxlhdjar77j6lg0wypcc9uar5d2shskhj42g";
        const ADDR_TEST: &str = "addr_test1xrphkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gt7r0vd4msrxnuwnccdxlhdjar77j6lg0wypcc9uar5d2shs4p04xh";

        let main = from_bech32(ADDR_MAIN);
        assert!(matches!(
            main,
            Address {
                payment: Credential::Script(SCRIPT_HASH),
                stake: Some(credential::Delegation::Script(SCRIPT_HASH),),
                network: Network::Main
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = from_bech32(ADDR_TEST);
        assert!(matches!(
            test,
            Address {
                payment: Credential::Script(SCRIPT_HASH),
                stake: Some(credential::Delegation::Script(SCRIPT_HASH),),
                network: Network::Test
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type4() {
        const ADDR_MAIN: &str =
            "addr1gx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer5pnz75xxcrzqf96k";
        const ADDR_TEST: &str =
            "addr_test1gz2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer5pnz75xxcrdw5vky";

        let main = from_bech32(ADDR_MAIN);
        assert!(matches!(
            main,
            Address {
                payment: Credential::VerificationKey(VK),
                stake: Some(credential::Delegation::Pointer(POINTER),),
                network: Network::Main
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = from_bech32(ADDR_TEST);
        assert!(matches!(
            test,
            Address {
                payment: Credential::VerificationKey(VK),
                stake: Some(credential::Delegation::Pointer(POINTER),),
                network: Network::Test
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type5() {
        const ADDR_MAIN: &str =
            "addr128phkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gtupnz75xxcrtw79hu";
        const ADDR_TEST: &str =
            "addr_test12rphkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gtupnz75xxcryqrvmw";

        let main = from_bech32(ADDR_MAIN);
        assert!(matches!(
            main,
            Address {
                payment: Credential::Script(SCRIPT_HASH),
                stake: Some(credential::Delegation::Pointer(POINTER)),
                network: Network::Main
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = from_bech32(ADDR_TEST);
        assert!(matches!(
            test,
            Address {
                payment: Credential::Script(SCRIPT_HASH),
                stake: Some(credential::Delegation::Pointer(POINTER)),
                network: Network::Test
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type6() {
        const ADDR_MAIN: &str = "addr1vx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzers66hrl8";
        const ADDR_TEST: &str = "addr_test1vz2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzerspjrlsz";

        let main = from_bech32(ADDR_MAIN);
        assert!(matches!(
            main,
            Address {
                payment: Credential::VerificationKey(VK),
                stake: None,
                network: Network::Main
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = from_bech32(ADDR_TEST);
        assert!(matches!(
            test,
            Address {
                payment: Credential::VerificationKey(VK),
                stake: None,
                network: Network::Test
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type7() {
        const ADDR_MAIN: &str = "addr1w8phkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gtcyjy7wx";
        const ADDR_TEST: &str = "addr_test1wrphkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gtcl6szpr";

        let main = from_bech32(ADDR_MAIN);
        assert!(matches!(
            main,
            Address {
                payment: Credential::Script(SCRIPT_HASH),
                stake: None,
                network: Network::Main
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = from_bech32(ADDR_TEST);
        assert!(matches!(
            test,
            Address {
                payment: Credential::Script(SCRIPT_HASH),
                stake: None,
                network: Network::Test
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type14() {
        const ADDR_MAIN: &str = "stake1uyehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gh6ffgw";
        const ADDR_TEST: &str = "stake_test1uqehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gssrtvn";

        let main = from_bech32(ADDR_MAIN);
        assert!(matches!(
            main,
            Account {
                credential: Credential::VerificationKey(STAKE_VK),
                network: Network::Main
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = from_bech32(ADDR_TEST);
        assert!(matches!(
            test,
            Account {
                credential: Credential::VerificationKey(STAKE_VK),
                network: Network::Test
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type15() {
        const ADDR_MAIN: &str = "stake178phkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gtcccycj5";
        const ADDR_TEST: &str = "stake_test17rphkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gtcljw6kf";

        let main = from_bech32(ADDR_MAIN);
        assert!(matches!(
            main,
            Account {
                credential: Credential::Script(SCRIPT_HASH),
                network: Network::Main
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = from_bech32(ADDR_TEST);
        assert!(matches!(
            test,
            Account {
                credential: Credential::Script(SCRIPT_HASH),
                network: Network::Test
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }
}
