pub mod header;

pub use header::Header;
use minicbor::Encode;

use crate::{transaction, witness};

#[derive(Debug, Clone, PartialEq, Eq, Encode)]
pub struct Block {
    #[n(0)]
    pub header: Header,
    #[n(1)]
    pub transaction_bodies: Box<[transaction::Body]>,
    #[n(2)]
    pub witness_sets: Box<[witness::Set]>,
    #[n(3)]
    pub auxiliary_data: Box<(u16, transaction::Data)>,
    #[n(4)]
    pub invalid_transactions: Box<[u16]>,
}
