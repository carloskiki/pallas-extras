use tinycbor_derive::{CborLen, Decode, Encode};

use crate::crypto;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Signature<'a> {
    certificate: super::Certificate<'a>,
    #[cbor(with = "cbor_util::Bytes<'a, crypto::Signature>")]
    signature: &'a crypto::Signature,
}
