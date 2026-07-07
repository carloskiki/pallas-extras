use tinycbor_derive::{CborLen, Decode, Encode};

/// A transaction payload.
///
/// Combines a transaction with its witnesses.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Payload<'a> {
    /// The transaction.
    pub transaction: super::Transaction,
    /// The witnesses for each input of the transaction.
    ///
    /// Each witness authenticates the input at the same index in the transaction.
    pub witnesses: Vec<super::Witness<'a>>,
}
