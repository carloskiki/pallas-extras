use crate::crypto::Blake2b256Digest;

pub struct Proof {
    transaction_count: u32,
    merkle_root: Blake2b256Digest,
    witness_hash: Blake2b256Digest,
}
