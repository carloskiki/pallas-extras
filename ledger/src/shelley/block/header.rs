use crate::crypto;

pub mod body;
pub use body::Body;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Header<'a> {
    body: Body<'a>,
    #[cbor(with = "cbor_util::Bytes<'a, crypto::kes::Signature>")]
    signature: &'a crypto::kes::Signature,
}
