use crate::crypto;
use std::hash::Hash;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Bootstrap<'a> {
    #[cbor(with = "cbor_util::VerifyingKey<'a>")]
    pub key: &'a crypto::VerifyingKey,
    #[cbor(with = "cbor_util::Signature<'a>")]
    pub signature: &'a crypto::Signature,
    pub chain_code: &'a [u8; 32],
    pub attributes: &'a [u8],
}

impl Hash for Bootstrap<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.key.0);
        state.write(&self.signature.to_bytes());
        self.chain_code.hash(state);
        self.attributes.hash(state);
    }
}
