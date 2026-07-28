use tinycbor::{Any, encoded::With};
use tinycbor_derive::{CborLen, Decode, Encode};

use crate::byron::{Update, delegation, transaction};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Body<'a> {
    /// The list of transactions in the block.
    ///
    /// Each transaction memoizes its CBOR encoding to ensure accurate validation. If the
    /// transactions were to be re-encoded differently, fees may change and the block would become
    /// invalid.
    pub transactions: Box<[With<'a, transaction::Payload<'a>>]>,
    pub ssc: Any<'a>,
    pub delegations: Box<[delegation::Certificate<'a>]>,
    pub update: Update<'a>,
}
