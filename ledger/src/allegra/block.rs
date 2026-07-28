use crate::{allegra, transaction::Index};
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod header;
pub use header::Header;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block<'a> {
    pub header: Header<'a>,
    pub transaction_bodies: Box<[super::transaction::Body<'a>]>,
    pub transaction_witness_sets: Box<[allegra::transaction::witness::Set<'a>]>,
    pub transaction_data: crate::Unique<Box<[(Index, allegra::transaction::Data<'a>)]>, false>,
}
