use crate::crypto;
use tinycbor_derive::{CborLen, Decode, Encode};

/// Witness data for a transaction input owned by a [`Redeem`](crate::byron::address::Type::Redeem) address.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Redeemer<'a> {
    /// The verifying key corresponding to the input address.
    #[cbor(with = "cbor_util::VerifyingKey<'a>")]
    pub key: &'a crypto::VerifyingKey,
    /// Signature authorizing the transaction to spend an input owned by the witnessed address.
    ///
    /// Specifically, this signs the transaction [`Id`](crate::transaction::Id) with the private key
    /// corresponding to [`Self::key`].
    #[cbor(with = "cbor_util::Signature<'a>")]
    pub signature: &'a crypto::Signature,
}
