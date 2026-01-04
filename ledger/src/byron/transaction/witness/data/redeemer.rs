use tinycbor_derive::{CborLen, Decode, Encode};
use crate::crypto;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Redeemer {
    pub key: crypto::VerifyingKey,
    #[cbor(with = "cbor_util::Signature<crypto::Signature>")]
    pub signature: crypto::Signature,
}
