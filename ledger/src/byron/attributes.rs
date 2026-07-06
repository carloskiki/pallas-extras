use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Attributes;
