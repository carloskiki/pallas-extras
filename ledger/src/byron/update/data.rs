use tinycbor_derive::{CborLen, Decode, Encode};

use crate::crypto::Blake2b256Digest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Data<'a> {
    diff_hash: &'a Blake2b256Digest,
    package_hash: &'a Blake2b256Digest,
    updater_hash: &'a Blake2b256Digest,
    markdown_hash: &'a Blake2b256Digest,
}
