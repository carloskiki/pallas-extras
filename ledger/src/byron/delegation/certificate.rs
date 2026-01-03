use crate::{crypto, epoch};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Certificate {
    epoch: epoch::Number,
    #[cbor(with = "cbor_util::ExtendedVerifyingKey")]
    issuer: crypto::ExtendedVerifyingKey,
    #[cbor(with = "cbor_util::ExtendedVerifyingKey")]
    delegate: crypto::ExtendedVerifyingKey,
    #[cbor(with = "cbor_util::Signature<crypto::Signature>")]
    signature: crypto::Signature,
}
