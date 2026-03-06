use std::hash::Hash;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct VerifyingKey<'a> {
    #[cbor(with = "cbor_util::VerifyingKey<'a>")]
    pub vkey: &'a crate::crypto::VerifyingKey,
    #[cbor(with = "cbor_util::Signature<'a>")]
    pub signature: &'a crate::crypto::Signature,
}

impl Hash for VerifyingKey<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.vkey.0);
        state.write(&self.signature.to_bytes());
    }
}
