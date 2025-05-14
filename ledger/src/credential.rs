use minicbor::{CborLen, Decode, Encode};

use crate::crypto::Blake2b224Digest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(flat)]
pub enum Credential {
    #[n(0)]
    Script(#[cbor(n(0), with = "minicbor::bytes")] Blake2b224Digest),
    #[n(1)]
    VerificationKey(#[cbor(n(0), with = "minicbor::bytes")] Blake2b224Digest),
}

impl AsRef<Blake2b224Digest> for Credential {
    fn as_ref(&self) -> &Blake2b224Digest {
        match self {
            Credential::Script(digest) | Credential::VerificationKey(digest) => digest,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Delegation {
    StakeKey(Blake2b224Digest),
    Script(Blake2b224Digest),
    Pointer(ChainPointer),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ChainPointer {
    pub slot: u64,
    pub tx_index: u64,
    pub cert_index: u64,
}

impl ChainPointer {
    pub(crate) fn from_bytes(bytes: impl IntoIterator<Item = u8>) -> Option<Self> {
        let mut cp = ChainPointer {
            slot: 0,
            tx_index: 0,
            cert_index: 0,
        };
        let mut bytes_iter = bytes.into_iter().peekable();
        let numbers = [&mut cp.slot, &mut cp.tx_index, &mut cp.cert_index];
        for num in numbers {
            bytes_iter.peek()?;
            for byte in bytes_iter.by_ref() {
                *num = (*num << 7) | (byte & 0x7f) as u64;
                if byte & 0x80 == 0 {
                    break;
                }
            }
        }
        Some(cp)
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
