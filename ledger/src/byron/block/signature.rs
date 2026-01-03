use tinycbor_derive::{CborLen, Decode, Encode};
use crate::{crypto, byron::delegation};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[allow(clippy::large_enum_variant)]
pub enum Signature {
    #[n(0)]
    Signature(#[cbor(with = "cbor_util::Signature<crypto::Signature>")] crypto::Signature),
    #[n(2)]
    Delegated(delegation::Signature),
}
