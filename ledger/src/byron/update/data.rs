use tinycbor_derive::{Encode, CborLen, Decode};

use crate::crypto::Blake2b256Digest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Data {
    diff_hash: Blake2b256Digest,
    package_hash: Blake2b256Digest,
    updater_hash: Blake2b256Digest,
    markdown_hash: Blake2b256Digest,
}
