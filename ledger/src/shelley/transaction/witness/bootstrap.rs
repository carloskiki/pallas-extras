use crate::crypto;
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
