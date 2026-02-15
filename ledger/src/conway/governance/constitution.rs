use super::Anchor;
use crate::crypto::Blake2b224Digest;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Constitution<'a> {
    pub anchor: Anchor<'a>,
    pub script_hash: Option<&'a Blake2b224Digest>,
}
