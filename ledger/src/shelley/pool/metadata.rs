use crate::crypto::Blake2b256Digest;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, CborLen, Encode, Decode)]
pub struct Metadata<'a> {
    pub url: &'a super::super::Url,
    pub hash: &'a Blake2b256Digest,
}
