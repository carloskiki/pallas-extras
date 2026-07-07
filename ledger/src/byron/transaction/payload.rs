use tinycbor::encoded::With;
use tinycbor_derive::{CborLen, Decode, Encode};

/// A transaction payload.
///
/// Combines a transaction with its witnesses.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Payload<'a> {
    /// The transaction.
    ///
    /// The transaction memoizes its bytes, because witnesses are computed from the transaction
    /// bytes. Re-encoding the transaction bytes could result in a different encoding, invalidating
    /// witnesses.
    pub transaction: With<'a, super::Transaction>,
    /// The witnesses for each input of the transaction.
    ///
    /// Each witness authenticates the input at the same index in the transaction.
    pub witnesses: Vec<super::Witness<'a>>,
}
