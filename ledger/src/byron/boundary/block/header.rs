use tinycbor_derive::{CborLen, Decode, Encode};
use crate::byron::Attributes;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Header<'a> {
    pub protocol_magic: u32,
    pub previous_block: crate::byron::block::Id,
    pub proof: crate::crypto::Blake2b256Digest,
    pub consensus_data: super::Data,
    pub extra_data: [Attributes<'a>; 1],
}
