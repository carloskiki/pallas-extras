use crate::{crypto, epoch};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Certificate<'a> {
    epoch: epoch::Number,
    #[cbor(with = "cbor_util::ExtendedVerifyingKey<'a>")]
    issuer: &'a crypto::ExtendedVerifyingKey,
    #[cbor(with = "cbor_util::ExtendedVerifyingKey<'a>")]
    delegate: &'a crypto::ExtendedVerifyingKey,
    #[cbor(with = "cbor_util::Signature<crypto::Signature>")]
    signature: crypto::Signature,
}
