use tinycbor_derive::{CborLen, Decode, Encode};
use crate::{crypto, byron::delegation};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Signature<'a> {
    #[n(0)]
    Signature(#[cbor(with = "cbor_util::Signature<'a>")] &'a crypto::Signature),
    #[n(2)]
    Delegated(delegation::Signature<'a>),
}
