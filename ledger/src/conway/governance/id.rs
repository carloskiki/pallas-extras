use tinycbor_derive::{CborLen, Decode, Encode};
use crate::conway::transaction;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Id<'a> {
    transaction_id: &'a transaction::Id,
    index: u16,
}
