use std::fmt::Display;
use std::error::Error;

use const_hex::{FromHex, FromHexError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Ensure that the entropy source is in between 16 and 32 bytes in length and also it is a
/// multiple of 4 bytes .
pub struct Entropy(pub(crate) [u8; 32], pub(crate) u8);

impl AsRef<[u8]> for Entropy {
    fn as_ref(&self) -> &[u8] {
        &self.0[..self.1 as usize]
    }
}

impl AsMut<[u8]> for Entropy {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0[..self.1 as usize]
    }
}

impl From<&[u8; 16]> for Entropy {
    fn from(entropy: &[u8; 16]) -> Self {
        let mut new = [0; 32];
        new[..16].copy_from_slice(entropy);
        Entropy(new, 16)
    }
}

impl From<&[u8; 20]> for Entropy {
    fn from(entropy: &[u8; 20]) -> Self {
        let mut new = [0; 32];
        new[..20].copy_from_slice(entropy);
        Entropy(new, 20)
    }
}

impl From<&[u8; 24]> for Entropy {
    fn from(entropy: &[u8; 24]) -> Self {
        let mut new = [0; 32];
        new[..24].copy_from_slice(entropy);
        Entropy(new, 24)
    }
}

impl From<&[u8; 28]> for Entropy {
    fn from(entropy: &[u8; 28]) -> Self {
        let mut new = [0; 32];
        new[..28].copy_from_slice(entropy);
        Entropy(new, 28)
    }
}

impl From<[u8; 32]> for Entropy {
    fn from(entropy: [u8; 32]) -> Self {
        Entropy(entropy, 32)
    }
}

impl FromHex for Entropy {
    type Error = FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let mut bytes = [0; 32];
        match hex.as_ref().len() {
            32 => {
                let hex_array = <[u8; 16]>::from_hex(hex)?;
                bytes[..16].copy_from_slice(&hex_array);
                Ok(Entropy(bytes, 16))
            }
            40 => {
                let hex_array = <[u8; 20]>::from_hex(hex)?;
                bytes[..20].copy_from_slice(&hex_array);
                Ok(Entropy(bytes, 20))
            }
            48 => {
                let hex_array = <[u8; 24]>::from_hex(hex)?;
                bytes[..24].copy_from_slice(&hex_array);
                Ok(Entropy(bytes, 24))
            }
            56 => {
                let hex_array = <[u8; 28]>::from_hex(hex)?;
                bytes[..28].copy_from_slice(&hex_array);
                Ok(Entropy(bytes, 28))
            }
            64 => {
                let bytes = <[u8; 32]>::from_hex(hex)?;
                Ok(Entropy(bytes, 32))
            }
            _ => Err(FromHexError::InvalidStringLength)
        }

        
    }
}

#[cfg(test)]
mod tests {
    use const_hex::ToHexExt;

    use super::*;

    const VALID_HEX: [&str; 5] = [
        "000102030405060708090a0b0c0d0e0f",
        "000102030405060708090a0b0c0d0e0f10111213",
        "000102030405060708090a0b0c0d0e0f1011121314151617",
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b",
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
    ];

    #[test]
    fn hex_roundtrip() {
        const EXPECTED_LEN: [usize; 5] = [16, 20, 24, 28, 32];
        const BYTES: [u8; 32] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ];

        for (hex, len) in VALID_HEX.iter().zip(EXPECTED_LEN) {
            let entropy = Entropy::from_hex(hex).unwrap();
            assert_eq!(entropy.as_ref().len(), len);
            assert_eq!(entropy.as_ref(), &BYTES[..len]);
            let generated_hex = entropy.encode_hex();
            assert_eq!(&generated_hex, hex);
        }
    }

    #[test]
    fn from_sizes() {
        const A16: [u8; 16] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        ];
        const A20: [u8; 20] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
        ];
        const A24: [u8; 24] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
            23,
        ];
        const A28: [u8; 28] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
            23, 24, 25, 26, 27,
        ];
        const A32: [u8; 32] = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
            23, 24, 25, 26, 27, 28, 29, 30, 31,
        ];
        let a16 = Entropy::from(&A16);
        let a20 = Entropy::from(&A20);
        let a24 = Entropy::from(&A24);
        let a28 = Entropy::from(&A28);
        let a32 = Entropy::from(A32);
        assert_eq!(a16.as_ref(), &A16);
        assert_eq!(a20.as_ref(), &A20);
        assert_eq!(a24.as_ref(), &A24);
        assert_eq!(a28.as_ref(), &A28);
        assert_eq!(a32.as_ref(), &A32);
    }
}
