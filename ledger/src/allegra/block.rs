use crate::{shelley::transaction::Index, allegra};
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod header;
pub use header::Header;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block<'a> {
    pub header: Header<'a>,
    pub transaction_bodies: Vec<super::transaction::Body<'a>>,
    pub transaction_witness_sets: Vec<allegra::transaction::witness::Set<'a>>,
    pub transaction_data: Vec<(Index, allegra::transaction::Data<'a>)>,
}
