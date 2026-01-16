use tinycbor_derive::{CborLen, Decode, Encode};
use crate::{crypto::Blake2b256Digest, shelley::Url};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, CborLen, Encode, Decode)]
pub struct Metadata<'a> {
    pub url: &'a Url,
    pub hash: &'a Blake2b256Digest,
}
