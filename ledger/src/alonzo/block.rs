use super::transaction;
use crate::{Unique, transaction::Index};
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod header;
pub use header::Header;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block<'a> {
    pub header: Header<'a>,
    pub transaction_bodies: Box<[transaction::Body<'a>]>,
    pub transaction_witness_sets: Box<[transaction::witness::Set<'a>]>,
    pub transaction_data: Unique<Box<[(Index, transaction::Data<'a>)]>, false>,
    pub invalid_transactions: Box<[Index]>,
}
