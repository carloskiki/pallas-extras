use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct VerifyingKey<'a> {
    #[cbor(with = "cbor_util::VerifyingKey<'a>")]
    pub vkey: &'a crate::crypto::VerifyingKey,
    #[cbor(with = "cbor_util::Signature<'a>")]
    pub signature: &'a crate::crypto::Signature,
}
