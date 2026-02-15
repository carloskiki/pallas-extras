use crate::crypto::Blake2b256Digest;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Proof<'a> {
    pub transaction_count: u32,
    pub merkle_root: &'a Blake2b256Digest,
    pub witnesses_hash: &'a Blake2b256Digest,
}
