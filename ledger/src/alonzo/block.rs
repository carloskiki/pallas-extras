use super::transaction;
use crate::shelley::transaction::Index;
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod header;
pub use header::Header;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block<'a> {
    pub header: Header<'a>,
    pub transaction_bodies: Vec<transaction::Body<'a>>,
    pub transaction_witness_sets: Vec<transaction::witness::Set<'a>>,
    pub transaction_data: Vec<(Index, transaction::Data<'a>)>,
    pub invalid_transactions: Vec<Index>,
}
