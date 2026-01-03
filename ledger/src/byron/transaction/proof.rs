use tinycbor_derive::{CborLen, Decode, Encode};
use crate::crypto::Blake2b256Digest;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Proof {
    pub transaction_count: u32,
    pub merkle_root: Blake2b256Digest,
    pub witnesses_hash: Blake2b256Digest,
}
