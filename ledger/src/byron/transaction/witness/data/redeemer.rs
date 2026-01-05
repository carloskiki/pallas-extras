use tinycbor_derive::{CborLen, Decode, Encode};
use crate::crypto;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Redeemer<'a> {
    pub key: &'a crypto::VerifyingKey,
    #[cbor(with = "cbor_util::Bytes<'a, crypto::Signature>")]
    pub signature: &'a crypto::Signature,
}
