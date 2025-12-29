use tinycbor_derive::{CborLen, Decode, Encode};

use crate::crypto::Blake2b256Digest;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub enum Input {
    #[n(0)]
    Input {
        id: Blake2b256Digest,
        index: u32,
    },
}
