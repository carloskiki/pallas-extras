use tinycbor_derive::{CborLen, Decode, Encode};
use crate::{conway::Url, crypto};


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Anchor<'a> {
    url: &'a Url,
    data_hash: &'a crypto::Blake2b256Digest,
}

