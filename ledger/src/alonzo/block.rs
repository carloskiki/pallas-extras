use super::transaction;
use crate::{Unique, transaction::Index};
use tinycbor::encoded::With;
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod header;
pub use header::Header;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block<'a> {
    pub header: With<'a, Header<'a>>,
    pub transaction_bodies: Vec<transaction::Body<'a>>,
    pub transaction_witness_sets: Vec<transaction::witness::Set<'a>>,
    pub transaction_data: Unique<Vec<(Index, transaction::Data<'a>)>, false>,
    pub invalid_transactions: Vec<Index>,
}
