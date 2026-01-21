use crate::crypto;
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod body;
pub use body::Body;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Header<'a> {
    pub body: Body<'a>,
    #[cbor(with = "cbor_util::Bytes<'a, crypto::kes::Signature>")]
    pub signature: &'a crypto::kes::Signature,
}
