use crate::{crypto::Blake2b256Digest, shelley::transaction};
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod header;
pub use header::Header;

pub type Number = u64;
pub type Size = u32;
pub type Id = Blake2b256Digest;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block<'a> {
    pub header: Header<'a>,
    pub transaction_bodies: Vec<transaction::Body<'a>>,
    pub transaction_witness_sets: Vec<transaction::witness::Set<'a>>,
    pub transaction_data: Vec<(transaction::Index, transaction::Data<'a>)>,
}
