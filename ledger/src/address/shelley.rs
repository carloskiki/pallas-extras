use std::{
    fmt::{Display, Write},
    iter,
    str::FromStr,
};

use bech32::{Bech32, ByteIterExt, Fe32IterExt, Hrp};
use minicbor::{Decode, Encode, decode, encode};

use crate::credential::{self, ChainPointerIter};
use crate::crypto::Blake2b224Digest;

const HASH_SIZE: usize = 28;

// TODO: We don't want to have Addresses have a MAINNET const, because that would impact all of the
// struct chain.
// - Use a program wide constant for which network we are on (mainnet, preview, preprod), and
// choose at build time with a feature flag.
// - Add the network const everywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address<const MAINNET: bool> {
    pub payment: credential::Payment,
    pub stake: Option<credential::Delegation>,
}

impl<const M: bool> Address<M> {
    fn from_bytes(bytes: impl IntoIterator<Item = u8>) -> Result<Self, AddressFromBytesError> {
        let mut data = bytes.into_iter();

        let first_byte = data.next().ok_or(AddressFromBytesError::TooShort)?;
        let header = first_byte >> 4;
        let network_magic = first_byte & 0b0000_1111;
        if network_magic != M as u8 {
            return Err(AddressFromBytesError::NetworkMagic);
        }
        let mut first_hash: Blake2b224Digest = Default::default();
        let mut len = 0;
        data.by_ref().zip(first_hash.iter_mut()).for_each(|(b, h)| {
            len += 1;
            *h = b;
        });
        if len != HASH_SIZE {
            return Err(AddressFromBytesError::TooShort);
        }

        if header < 0b0100 {
            let mut second_hash: Blake2b224Digest = Default::default();
            len = 0;
            data.by_ref()
                .zip(second_hash.iter_mut())
                .for_each(|(b, h)| {
                    len += 1;
                    *h = b;
                });
            if len != HASH_SIZE || data.next().is_some() {
                return Err(AddressFromBytesError::TooShort);
            }

            let (payment, stake) = match header {
                0b0000 => (
                    credential::Payment::VerificationKey(first_hash),
                    credential::Delegation::StakeKey(second_hash),
                ),
                0b0001 => (
                    credential::Payment::Script(first_hash),
                    credential::Delegation::StakeKey(second_hash),
                ),
                0b0010 => (
                    credential::Payment::VerificationKey(first_hash),
                    credential::Delegation::Script(second_hash),
                ),
                0b0011 => (
                    credential::Payment::Script(first_hash),
                    credential::Delegation::Script(second_hash),
                ),
                _ => unreachable!(),
            };
            Ok(Address {
                payment,
                stake: Some(stake),
            })
        } else if header < 0b0110 {
            let pointer = credential::ChainPointer::from_bytes(data.by_ref())
                .ok_or(AddressFromBytesError::ChainPointer)?;
            let payment = match header {
                0b0100 => credential::Payment::VerificationKey(first_hash),
                0b0101 => credential::Payment::Script(first_hash),
                _ => unreachable!(),
            };

            Ok(Address {
                payment,
                stake: Some(credential::Delegation::Pointer(pointer)),
            })
        } else if header < 0b1000 {
            if data.next().is_some() {
                return Err(AddressFromBytesError::TooShort);
            }

            let payment = match header {
                0b0110 => credential::Payment::VerificationKey(first_hash),
                0b0111 => credential::Payment::Script(first_hash),
                _ => unreachable!(),
            };

            Ok(Address {
                payment,
                stake: None,
            })
        } else {
            Err(AddressFromBytesError::AddressType)
        }
    }
}

impl<C, const M: bool> Encode<C> for Address<M> {
    fn encode<W: encode::Write>(
        &self,
        e: &mut encode::Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        let bytes = self.into_iter().collect::<Box<[_]>>();
        e.bytes(&bytes)?.ok()
    }
}

impl<'b, C, const M: bool> Decode<'b, C> for Address<M> {
    fn decode(d: &mut decode::Decoder<'b>, _: &mut C) -> Result<Self, decode::Error> {
        // This ignores decoding errors of the inner slices, but should not matter because if the
        // inner slice errors then the value wont parse correctly anyway.
        // This may make use vulnerable to DoS, but at this point having an application on the
        // internet is a DoS vector.
        let data = d.bytes_iter()?.flatten().flatten().copied();

        Address::<M>::from_bytes(data).map_err(decode::Error::custom)
    }
}

impl<const M: bool> Display for Address<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hrp = Hrp::parse_unchecked(if M { "addr" } else { "addr_test" });
        self.into_iter()
            .bytes_to_fes()
            .with_checksum::<Bech32>(&hrp)
            .chars()
            .try_for_each(|c| f.write_char(c))
    }
}

impl<'a, const M: bool> IntoIterator for &'a Address<M> {
    type Item = u8;

    type IntoIter = iter::Chain<
        iter::Chain<iter::Once<u8>, iter::Copied<core::slice::Iter<'a, u8>>>,
        either::Either<
            either::Either<iter::Copied<core::slice::Iter<'a, u8>>, ChainPointerIter>,
            iter::Empty<u8>,
        >,
    >;

    fn into_iter(self) -> Self::IntoIter {
        let header = match (self.payment, self.stake) {
            (
                credential::Payment::VerificationKey(_),
                Some(credential::Delegation::StakeKey(_)),
            ) => 0b0000,
            (credential::Payment::Script(_), Some(credential::Delegation::StakeKey(_))) => 0b0001,
            (credential::Payment::VerificationKey(_), Some(credential::Delegation::Script(_))) => {
                0b0010
            }
            (credential::Payment::Script(_), Some(credential::Delegation::Script(_))) => 0b0011,
            (credential::Payment::VerificationKey(_), Some(credential::Delegation::Pointer(_))) => {
                0b0100
            }
            (credential::Payment::Script(_), Some(credential::Delegation::Pointer(_))) => 0b0101,
            (credential::Payment::VerificationKey(_), None) => 0b0110,
            (credential::Payment::Script(_), None) => 0b0111,
        };
        let network_magic = M as u8;
        let first_byte = (header << 4) | network_magic;

        iter::once(first_byte)
            .chain(self.payment.as_ref().iter().copied())
            .chain(match self.stake {
                Some(
                    credential::Delegation::StakeKey(ref hash)
                    | credential::Delegation::Script(ref hash),
                ) => either::Left(either::Left(hash.iter().copied())),
                Some(credential::Delegation::Pointer(pointer)) => {
                    either::Left(either::Right(pointer.into_iter()))
                }
                None => either::Right(iter::empty()),
            })
    }
}

impl<const M: bool> FromStr for Address<M> {
    type Err = AddressFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, data) = bech32::decode(s).map_err(AddressFromStrError::Bech32)?;

        Address::<M>::from_bytes(data).map_err(AddressFromStrError::from)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StakeAddress<const MAINNET: bool> {
    pub credential: credential::Payment,
}

impl<const M: bool> StakeAddress<M> {
    pub fn from_bytes(bytes: impl IntoIterator<Item = u8>) -> Result<Self, AddressFromBytesError> {
        let mut data = bytes.into_iter();
        let first_byte = data.next().ok_or(AddressFromBytesError::TooShort)?;
        let header = first_byte >> 4;
        let network_magic = first_byte & 0b0000_1111;
        if network_magic != M as u8 {
            return Err(AddressFromBytesError::NetworkMagic);
        }

        let mut hash: Blake2b224Digest = Default::default();
        let mut len = 0;
        data.take(HASH_SIZE)
            .zip(hash.iter_mut())
            .for_each(|(b, h)| {
                len += 1;
                *h = b;
            });
        if len != HASH_SIZE {
            return Err(AddressFromBytesError::TooShort);
        }

        let credential = if header == 0b1110 {
            credential::Payment::VerificationKey(hash)
        } else if header == 0b1111 {
            credential::Payment::Script(hash)
        } else {
            return Err(AddressFromBytesError::AddressType);
        };

        Ok(StakeAddress { credential })
    }
}

impl<const M: bool> Display for StakeAddress<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hrp = Hrp::parse_unchecked(if M { "stake" } else { "stake_test" });

        self.into_iter()
            .bytes_to_fes()
            .with_checksum::<Bech32>(&hrp)
            .chars()
            .try_for_each(|c| f.write_char(c))
    }
}

impl<'a, const M: bool> IntoIterator for &'a StakeAddress<M> {
    type Item = u8;

    type IntoIter =
        core::iter::Chain<core::iter::Once<u8>, core::iter::Copied<core::slice::Iter<'a, u8>>>;

    fn into_iter(self) -> Self::IntoIter {
        let header = match self.credential {
            credential::Payment::VerificationKey(_) => 0b1110,
            credential::Payment::Script(_) => 0b1111,
        };
        let network_magic = M as u8;
        let first_byte = (header << 4) | network_magic;

        core::iter::once(first_byte).chain(self.credential.as_ref().iter().copied())
    }
}

impl<const M: bool> FromStr for StakeAddress<M> {
    type Err = AddressFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, data) = bech32::decode(s).map_err(AddressFromStrError::Bech32)?;

        StakeAddress::<M>::from_bytes(data).map_err(AddressFromStrError::from)
    }
}

impl<C, const M: bool> Encode<C> for StakeAddress<M> {
    fn encode<W: encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        let bytes = self.into_iter().collect::<Box<[_]>>();
        e.bytes(&bytes)?.ok()
    }
}

impl<'b, C, const M: bool> Decode<'b, C> for StakeAddress<M> {
    fn decode(d: &mut minicbor::Decoder<'b>, _: &mut C) -> Result<Self, decode::Error> {
        // This ignores decoding errors of the inner slices, but should not matter because if the
        // inner slice errors then the value wont parse correctly anyway.
        // This may make use vulnerable to DoS, but at this point having an application on the
        // internet is a DoS vector.
        let data = d.bytes_iter()?.flatten().flatten().copied();

        StakeAddress::<M>::from_bytes(data).map_err(decode::Error::custom)
    }
}

#[derive(Debug, Clone)]
pub enum AddressFromStrError {
    Bytes(AddressFromBytesError),
    /// Invalid bech32 encoding.
    Bech32(bech32::DecodeError),
}

impl From<bech32::DecodeError> for AddressFromStrError {
    fn from(value: bech32::DecodeError) -> Self {
        AddressFromStrError::Bech32(value)
    }
}

impl From<AddressFromBytesError> for AddressFromStrError {
    fn from(value: AddressFromBytesError) -> Self {
        AddressFromStrError::Bytes(value)
    }
}

impl Display for AddressFromStrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressFromStrError::Bytes(e) => write!(f, "Invalid address bytes: {}", e),
            AddressFromStrError::Bech32(e) => write!(f, "Invalid bech32 encoding: {}", e),
        }
    }
}

impl core::error::Error for AddressFromStrError {}

#[derive(Debug, Clone)]
pub enum AddressFromBytesError {
    /// The given input is too short.
    TooShort,
    /// The given input is too long.
    TooLong,
    /// Incorrect network magic.
    NetworkMagic,
    /// The header contains an invalid address type.
    AddressType,
    /// Error decoding the chain pointer.
    ChainPointer,
}

impl Display for AddressFromBytesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressFromBytesError::TooShort => write!(f, "The given input is too short"),
            AddressFromBytesError::TooLong => write!(f, "The given input is too long"),
            AddressFromBytesError::NetworkMagic => write!(f, "Incorrect network magic"),
            AddressFromBytesError::AddressType => {
                write!(f, "The header contains an invalid address type")
            }
            AddressFromBytesError::ChainPointer => write!(f, "Error decoding the chain pointer"),
        }
    }
}

impl core::error::Error for AddressFromBytesError {}

#[cfg(test)]
mod tests {
    //! All tests are coming from CIP 19

    use super::*;

    const VK: Blake2b224Digest = [
        148, 147, 49, 92, 217, 46, 181, 216, 196, 48, 78, 103, 183, 225, 106, 227, 109, 97, 211,
        69, 2, 105, 70, 87, 129, 26, 44, 142,
    ];
    const STAKE_VK: Blake2b224Digest = [
        51, 123, 98, 207, 255, 100, 3, 160, 106, 58, 203, 195, 79, 140, 70, 0, 60, 105, 254, 121,
        163, 98, 140, 239, 169, 196, 114, 81,
    ];
    const SCRIPT_HASH: Blake2b224Digest = [
        195, 123, 27, 93, 192, 102, 159, 29, 60, 97, 166, 253, 219, 46, 143, 222, 150, 190, 135,
        184, 129, 198, 11, 206, 142, 141, 84, 47,
    ];
    const POINTER: credential::ChainPointer = credential::ChainPointer {
        slot: 2498243,
        tx_index: 27,
        cert_index: 3,
    };

    #[test]
    fn type0() {
        const ADDR_MAIN: &str = "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x";
        const ADDR_TEST: &str = "addr_test1qz2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgs68faae";

        let main = Address::<true>::from_str(ADDR_MAIN).unwrap();
        assert!(matches!(
            main,
            Address {
                payment: credential::Payment::VerificationKey(VK),
                stake: Some(credential::Delegation::StakeKey(STAKE_VK))
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address {
                payment: credential::Payment::VerificationKey(VK),
                stake: Some(credential::Delegation::StakeKey(STAKE_VK))
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type1() {
        const ADDR_MAIN: &str = "addr1z8phkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gten0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgs9yc0hh";
        const ADDR_TEST: &str = "addr_test1zrphkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gten0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgsxj90mg";

        let main = Address::<true>::from_str(ADDR_MAIN).unwrap();
        assert!(matches!(
            main,
            Address {
                payment: credential::Payment::Script(SCRIPT_HASH),
                stake: Some(credential::Delegation::StakeKey(STAKE_VK))
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address {
                payment: credential::Payment::Script(SCRIPT_HASH),
                stake: Some(credential::Delegation::StakeKey(STAKE_VK))
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type2() {
        const ADDR_MAIN: &str = "addr1yx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzerkr0vd4msrxnuwnccdxlhdjar77j6lg0wypcc9uar5d2shs2z78ve";
        const ADDR_TEST: &str = "addr_test1yz2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzerkr0vd4msrxnuwnccdxlhdjar77j6lg0wypcc9uar5d2shsf5r8qx";

        let main = Address::<true>::from_str(ADDR_MAIN).unwrap();
        assert!(matches!(
            main,
            Address {
                payment: credential::Payment::VerificationKey(VK),
                stake: Some(credential::Delegation::Script(SCRIPT_HASH),)
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            main,
            Address {
                payment: credential::Payment::VerificationKey(VK),
                stake: Some(credential::Delegation::Script(SCRIPT_HASH),)
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type3() {
        const ADDR_MAIN: &str = "addr1x8phkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gt7r0vd4msrxnuwnccdxlhdjar77j6lg0wypcc9uar5d2shskhj42g";
        const ADDR_TEST: &str = "addr_test1xrphkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gt7r0vd4msrxnuwnccdxlhdjar77j6lg0wypcc9uar5d2shs4p04xh";

        let main = Address::<true>::from_str(ADDR_MAIN).unwrap();
        assert!(matches!(
            main,
            Address {
                payment: credential::Payment::Script(SCRIPT_HASH),
                stake: Some(credential::Delegation::Script(SCRIPT_HASH),)
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address {
                payment: credential::Payment::Script(SCRIPT_HASH),
                stake: Some(credential::Delegation::Script(SCRIPT_HASH),)
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

        let main = Address::<true>::from_str(ADDR_MAIN).unwrap();
        assert!(matches!(
            main,
            Address {
                payment: credential::Payment::VerificationKey(VK),
                stake: Some(credential::Delegation::Pointer(POINTER),)
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address {
                payment: credential::Payment::VerificationKey(VK),
                stake: Some(credential::Delegation::Pointer(POINTER),)
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

        let main = Address::<true>::from_str(ADDR_MAIN).unwrap();
        assert!(matches!(
            main,
            Address {
                payment: credential::Payment::Script(SCRIPT_HASH),
                stake: Some(credential::Delegation::Pointer(POINTER))
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address {
                payment: credential::Payment::Script(SCRIPT_HASH),
                stake: Some(credential::Delegation::Pointer(POINTER))
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type6() {
        const ADDR_MAIN: &str = "addr1vx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzers66hrl8";
        const ADDR_TEST: &str = "addr_test1vz2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzerspjrlsz";

        let main = Address::<true>::from_str(ADDR_MAIN).unwrap();
        assert!(matches!(
            main,
            Address {
                payment: credential::Payment::VerificationKey(VK),
                stake: None
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address {
                payment: credential::Payment::VerificationKey(VK),
                stake: None,
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type7() {
        const ADDR_MAIN: &str = "addr1w8phkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gtcyjy7wx";
        const ADDR_TEST: &str = "addr_test1wrphkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gtcl6szpr";

        let main = Address::<true>::from_str(ADDR_MAIN).unwrap();
        assert!(matches!(
            main,
            Address {
                payment: credential::Payment::Script(SCRIPT_HASH),
                stake: None
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address {
                payment: credential::Payment::Script(SCRIPT_HASH),
                stake: None
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type14() {
        const ADDR_MAIN: &str = "stake1uyehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gh6ffgw";
        const ADDR_TEST: &str = "stake_test1uqehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gssrtvn";

        let main = StakeAddress::<true>::from_str(ADDR_MAIN).unwrap();
        assert!(matches!(
            main,
            StakeAddress {
                credential: credential::Payment::VerificationKey(STAKE_VK)
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = StakeAddress::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            StakeAddress {
                credential: credential::Payment::VerificationKey(STAKE_VK)
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type15() {
        const ADDR_MAIN: &str = "stake178phkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gtcccycj5";
        const ADDR_TEST: &str = "stake_test17rphkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gtcljw6kf";

        let main = StakeAddress::<true>::from_str(ADDR_MAIN).unwrap();
        assert!(matches!(
            main,
            StakeAddress {
                credential: credential::Payment::Script(SCRIPT_HASH)
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = StakeAddress::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            StakeAddress {
                credential: credential::Payment::Script(SCRIPT_HASH)
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }
}
