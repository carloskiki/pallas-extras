use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Attributes {
    #[cbor(n(1), optional)]
    pub key_derivation_path: Option<Box<[u8]>>,
}
