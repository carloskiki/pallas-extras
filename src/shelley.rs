use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use bech32::{Bech32, ByteIterExt, Fe32IterExt, Hrp};

use crate::{Blake2b224Digest, Blake2b256Digest};

pub mod transaction;
pub mod witness;
pub mod block;
pub mod protocol;

const HASH_SIZE: usize = 28;

pub struct Nonce {
    pub variant: NonceVariant,
    pub nonce: Blake2b256Digest,
}

pub enum NonceVariant {
    NeutralNonce,
    Nonce,
}

pub struct RealNumber {
    pub numerator: u64,
    pub denominator: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address<const MAINNET: bool> {
    pub payment: PaymentCredential,
    pub stake: Option<DelegationCredential>,
}

impl<const M: bool> Display for Address<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_address::<M>(false, &self.payment, self.stake.as_ref(), f)
    }
}

#[derive(Debug)]
pub enum AddressFromStrError {
    /// The given input is too short.
    TooShort,
    /// The given input is too long.
    TooLong,
    /// Incorrect network magic.
    NetworkMagic,
    /// The header contains an invalid address type.
    AddressType,
    /// Invalid bech32 encoding.
    Bech32(bech32::DecodeError),
}

impl From<bech32::DecodeError> for AddressFromStrError {
    fn from(value: bech32::DecodeError) -> Self {
        AddressFromStrError::Bech32(value)
    }
}

impl<const M: bool> FromStr for Address<M> {
    type Err = AddressFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, data) = bech32::decode(s).map_err(AddressFromStrError::Bech32)?;

        let first_byte = data.first().ok_or(AddressFromStrError::TooShort)?;
        let header = first_byte >> 4;
        let network_magic = first_byte & 0b0000_1111;
        if network_magic != M as u8 {
            return Err(AddressFromStrError::NetworkMagic);
        }
        if data.len() < 1 + HASH_SIZE {
            return Err(AddressFromStrError::TooShort);
        }
        let first_hash = Blake2b224Digest::try_from(&data[1..1 + HASH_SIZE]).unwrap();

        if header < 0b0100 {
            if data.len() != 1 + 2 * HASH_SIZE {
                return Err(AddressFromStrError::TooShort);
            }
            let second_hash = Blake2b224Digest::try_from(&data[1 + HASH_SIZE..]).unwrap();
            let (payment, stake) = match header {
                0b0000 => (
                    PaymentCredential::VerificationKey(first_hash),
                    DelegationCredential::StakeKey(second_hash),
                ),
                0b0001 => (
                    PaymentCredential::Script(first_hash),
                    DelegationCredential::StakeKey(second_hash),
                ),
                0b0010 => (
                    PaymentCredential::VerificationKey(first_hash),
                    DelegationCredential::Script(second_hash),
                ),
                0b0011 => (
                    PaymentCredential::Script(first_hash),
                    DelegationCredential::Script(second_hash),
                ),
                _ => unreachable!(),
            };
            Ok(Address { payment, stake: Some(stake) })
        } else if header < 0b0110 {
            let pointer = ChainPointer::try_from(&data[1 + HASH_SIZE..]).unwrap();
            let payment = match header {
                0b0100 => PaymentCredential::VerificationKey(first_hash),
                0b0101 => PaymentCredential::Script(first_hash),
                _ => unreachable!(),
            };

            Ok(Address {
                payment,
                stake: Some(DelegationCredential::Pointer(pointer)),
            })
        } else if header < 0b1000 {
            if data.len() != 1 + HASH_SIZE {
                return Err(AddressFromStrError::TooShort);
            }
            
            let payment = match header {
                0b0110 => PaymentCredential::VerificationKey(first_hash),
                0b0111 => PaymentCredential::Script(first_hash),
                _ => unreachable!(),
            };

            Ok(Address { payment, stake: None })
        } else {
            Err(AddressFromStrError::AddressType)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StakeAddress<const MAINNET: bool> {
    pub credential: PaymentCredential,
}

impl<const M: bool> Display for StakeAddress<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_address::<M>(true, &self.credential, None, f)
    }
    
}

impl<const M: bool> FromStr for StakeAddress<M> {
    type Err = AddressFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, data) = bech32::decode(s).map_err(AddressFromStrError::Bech32)?;

        let first_byte = data.first().ok_or(AddressFromStrError::TooShort)?;
        let header = first_byte >> 4;
        let network_magic = first_byte & 0b0000_1111;
        if network_magic != M as u8 {
            return Err(AddressFromStrError::NetworkMagic);
        }
        if data.len() != 1 + HASH_SIZE {
            return Err(AddressFromStrError::TooShort);
        }
        let hash = Blake2b224Digest::try_from(&data[1..1 + HASH_SIZE]).unwrap();
        
        let credential = if header == 0b1110 {
            PaymentCredential::VerificationKey(hash)
        } else if header == 0b1111  {
            PaymentCredential::Script(hash)
        } else {
            return Err(AddressFromStrError::AddressType);
        };
        
        Ok(StakeAddress { credential })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaymentCredential {
    Script(Blake2b224Digest),
    VerificationKey(Blake2b224Digest),
}

impl AsRef<[u8; HASH_SIZE]> for PaymentCredential {
    fn as_ref(&self) -> &[u8; HASH_SIZE] {
        match self {
            PaymentCredential::Script(digest) | PaymentCredential::VerificationKey(digest) => {
                digest
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DelegationCredential {
    StakeKey(Blake2b224Digest),
    Script(Blake2b224Digest),
    Pointer(ChainPointer),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChainPointer {
    pub slot: u64,
    pub tx_index: u64,
    pub cert_index: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChainPointerDecodeError;

impl TryFrom<&[u8]> for ChainPointer {
    type Error = ChainPointerDecodeError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut cp = ChainPointer {
            slot: 0,
            tx_index: 0,
            cert_index: 0,
        };
        let mut bytes_iter = value.iter().copied();
        let numbers = [&mut cp.slot, &mut cp.tx_index, &mut cp.cert_index];
        for num in numbers {
            for byte in bytes_iter.by_ref() {
                *num = (*num << 7) | (byte & 0x7f) as u64;
                if byte & 0x80 == 0 {
                    break;
                }
            }
        }
        Ok(cp)
    }
}

impl IntoIterator for ChainPointer {
    type Item = u8;

    type IntoIter = ChainPointerIter;

    fn into_iter(self) -> Self::IntoIter {
        ChainPointerIter {
            slot: self.slot,
            tx_index: self.tx_index,
            cert_index: self.cert_index,
        }
    }
}

pub struct ChainPointerIter {
    slot: u64,
    tx_index: u64,
    cert_index: u64,
}

impl Iterator for ChainPointerIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let num = if self.slot != 0 {
            &mut self.slot
        } else if self.tx_index != 0 {
            &mut self.tx_index
        } else if self.cert_index != 0 {
            &mut self.cert_index
        } else {
            return None;
        };

        let bit_count = 64 - num.leading_zeros();
        // Get the first 7 bits in the correct window.
        // We do (- 1) because if there is a multiple of 7 bits, we don't want to shift by the
        // bitcount.
        let shift_value = (bit_count - 1) / 7 * 7;
        let mut value = *num >> shift_value;
        let mask = (1 << shift_value) - 1;
        *num &= mask;
        if *num != 0 {
            value |= 0x80;
        }
        Some(value as u8)
    }
}

fn display_address<const M: bool>(
    is_stake: bool,
    payment: &PaymentCredential,
    stake: Option<&DelegationCredential>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let hrp = if is_stake {
        Hrp::parse_unchecked(if M { "stake" } else { "stake_test" })
    } else {
        Hrp::parse_unchecked(if M { "addr" } else { "addr_test" })
    };

    let header = match (payment, stake, is_stake) {
        (PaymentCredential::VerificationKey(_), Some(DelegationCredential::StakeKey(_)), false) => {
            0b0000
        }
        (PaymentCredential::Script(_), Some(DelegationCredential::StakeKey(_)), false) => 0b0001,
        (PaymentCredential::VerificationKey(_), Some(DelegationCredential::Script(_)), false) => {
            0b0010
        }
        (PaymentCredential::Script(_), Some(DelegationCredential::Script(_)), false) => 0b0011,
        (PaymentCredential::VerificationKey(_), Some(DelegationCredential::Pointer(_)), false) => {
            0b0100
        }
        (PaymentCredential::Script(_), Some(DelegationCredential::Pointer(_)), false) => 0b0101,
        (PaymentCredential::VerificationKey(_), None, false) => 0b0110,
        (PaymentCredential::Script(_), None, false) => 0b0111,
        (PaymentCredential::VerificationKey(_), None, true) => 0b1110,
        (PaymentCredential::Script(_), None, true) => 0b1111,
        _ => unreachable!("Wrong combination of payment and stake credentials"),
    };
    let network_magic = M as u8;
    let first_byte = (header << 4) | network_magic;

    if let Some(stake) = stake {
        let first_part = std::iter::once(first_byte).chain(payment.as_ref().iter().copied());
        match stake {
            DelegationCredential::StakeKey(hash) | DelegationCredential::Script(hash) => first_part
                .chain(hash.iter().copied())
                .bytes_to_fes()
                .with_checksum::<Bech32>(&hrp)
                .chars()
                .try_for_each(|c| f.write_char(c)),
            DelegationCredential::Pointer(pointer) => first_part
                .chain(*pointer)
                .bytes_to_fes()
                .with_checksum::<Bech32>(&hrp)
                .chars()
                .try_for_each(|c| f.write_char(c)),
        }
    } else {
        std::iter::once(first_byte)
            .chain(payment.as_ref().iter().copied())
            .bytes_to_fes()
            .with_checksum::<Bech32>(&hrp)
            .chars()
            .try_for_each(|c| f.write_char(c))
    }
}

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
    const POINTER: ChainPointer = ChainPointer {
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
            Address::Ordinary {
                payment: PaymentCredential::VerificationKey(VK),
                stake: DelegationCredential::StakeKey(STAKE_VK)
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address::Ordinary {
                payment: PaymentCredential::VerificationKey(VK),
                stake: DelegationCredential::StakeKey(STAKE_VK)
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
            Address::Ordinary {
                payment: PaymentCredential::Script(SCRIPT_HASH),
                stake: DelegationCredential::StakeKey(STAKE_VK)
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address::Ordinary {
                payment: PaymentCredential::Script(SCRIPT_HASH),
                stake: DelegationCredential::StakeKey(STAKE_VK)
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
            Address::Ordinary {
                payment: PaymentCredential::VerificationKey(VK),
                stake: DelegationCredential::Script(SCRIPT_HASH),
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            main,
            Address::Ordinary {
                payment: PaymentCredential::VerificationKey(VK),
                stake: DelegationCredential::Script(SCRIPT_HASH),
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
            Address::Ordinary {
                payment: PaymentCredential::Script(SCRIPT_HASH),
                stake: DelegationCredential::Script(SCRIPT_HASH),
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address::Ordinary {
                payment: PaymentCredential::Script(SCRIPT_HASH),
                stake: DelegationCredential::Script(SCRIPT_HASH),
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
            Address::Ordinary {
                payment: PaymentCredential::VerificationKey(VK),
                stake: DelegationCredential::Pointer(POINTER),
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address::Ordinary {
                payment: PaymentCredential::VerificationKey(VK),
                stake: DelegationCredential::Pointer(POINTER),
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
            Address::Ordinary {
                payment: PaymentCredential::Script(SCRIPT_HASH),
                stake: DelegationCredential::Pointer(POINTER)
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address::Ordinary {
                payment: PaymentCredential::Script(SCRIPT_HASH),
                stake: DelegationCredential::Pointer(POINTER)
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
            Address::Enterprise {
                payment: PaymentCredential::VerificationKey(VK)
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address::Enterprise {
                payment: PaymentCredential::VerificationKey(VK)
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
            Address::Enterprise {
                payment: PaymentCredential::Script(SCRIPT_HASH)
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address::Enterprise {
                payment: PaymentCredential::Script(SCRIPT_HASH)
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type14() {
        const ADDR_MAIN: &str = "stake1uyehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gh6ffgw";
        const ADDR_TEST: &str = "stake_test1uqehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gssrtvn";

        let main = Address::<true>::from_str(ADDR_MAIN).unwrap();
        assert!(matches!(
            main,
            Address::Stake {
                credential: PaymentCredential::VerificationKey(STAKE_VK)
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address::Stake {
                credential: PaymentCredential::VerificationKey(STAKE_VK)
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }

    #[test]
    fn type15() {
        const ADDR_MAIN: &str = "stake178phkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gtcccycj5";
        const ADDR_TEST: &str = "stake_test17rphkx6acpnf78fuvxn0mkew3l0fd058hzquvz7w36x4gtcljw6kf";

        let main = Address::<true>::from_str(ADDR_MAIN).unwrap();
        assert!(matches!(
            main,
            Address::Stake {
                credential: PaymentCredential::Script(SCRIPT_HASH)
            }
        ));
        let serialized = main.to_string();
        assert_eq!(serialized, ADDR_MAIN);

        let test = Address::<false>::from_str(ADDR_TEST).unwrap();
        assert!(matches!(
            test,
            Address::Stake {
                credential: PaymentCredential::Script(SCRIPT_HASH)
            }
        ));
        let serialized = test.to_string();
        assert_eq!(serialized, ADDR_TEST);
    }
}
