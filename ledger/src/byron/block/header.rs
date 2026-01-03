use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Header {
    pub protocol_magic: u32,
    pub previous_block: super::Id,
    pub proof: super::Proof,
    pub consensus_data: super::consensus::Data,
    pub extra_data: super::Data,
}
