use crate::byron::block;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Header<'a> {
    pub protocol_magic: u32,
    pub previous_block: &'a block::Id,
    pub proof: super::Proof<'a>,
    pub consensus_data: super::data::consensus::Data<'a>,
    pub extra_data: super::data::extra::Data<'a>,
}
