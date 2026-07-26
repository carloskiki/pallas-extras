//! Transaction witness.
//!
//! A transaction witness both authenticates the transaction issuer and ensures integrity of the
//! transaction. The cryptographic signature found in the

use tinycbor_derive::{CborLen, Decode, Encode};

/// Data found in a witness.
pub mod data;

/// Witness for an input in a transaction.
///
/// Authenticates the transaction input holder. A valid witness for an input can only be produced
/// from knowledge of the private key corresponding to the input's address.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Witness<'a> {
    /// Witness for an input owned by a verifying-key-based address.
    #[n(0)]
    VerifyingKey(
        #[cbor(with = "tinycbor::Encoded<data::VerifyingKey<'a>>")] data::VerifyingKey<'a>,
    ),
    /// Witness for an input owned by a redeemer address.
    #[n(2)]
    Redeemer(#[cbor(with = "tinycbor::Encoded<data::Redeemer<'a>>")] data::Redeemer<'a>),
}
