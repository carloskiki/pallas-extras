use crate::{byron::crypto::ExtendedVerifyingKey, crypto::Signature, epoch};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Certificate<'a> {
    pub epoch: epoch::Number,
    #[cbor(with = "cbor_util::ExtendedVerifyingKey<'a>")]
    pub issuer: &'a ExtendedVerifyingKey,
    #[cbor(with = "cbor_util::ExtendedVerifyingKey<'a>")]
    pub delegate: &'a ExtendedVerifyingKey,
    #[cbor(with = "cbor_util::Signature<'a>")]
    pub signature: &'a Signature,
}
