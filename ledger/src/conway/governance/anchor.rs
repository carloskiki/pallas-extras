use tinycbor_derive::{CborLen, Decode, Encode};
use crate::{conway::url::Url, crypto};


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Anchor<'a> {
    url: Url<'a>,
    data_hash: &'a crypto::Blake2b256Digest,
}

