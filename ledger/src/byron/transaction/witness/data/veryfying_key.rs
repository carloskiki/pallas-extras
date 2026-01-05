use tinycbor_derive::{CborLen, Decode, Encode};
use crate::crypto;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct VerifyingKey<'a> {
    #[cbor(with = "cbor_util::ExtendedVerifyingKey<'a>")]
    pub key: &'a crypto::ExtendedVerifyingKey,
    #[cbor(with = "cbor_util::Bytes<'a, crypto::Signature>")]
    pub signature: &'a crypto::Signature,
}

