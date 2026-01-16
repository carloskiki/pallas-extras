use tinycbor_derive::{CborLen, Decode, Encode};
use crate::{conway::url::Url, crypto::Blake2b256Digest};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Metadata<'a> {
    pub url: Url<'a>,
    pub metadata: &'a Blake2b256Digest,
}

