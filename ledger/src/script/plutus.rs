use minicbor::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Encode, Decode, CborLen)]
#[cbor(transparent)]
pub struct Script(#[cbor(with = "minicbor::bytes")] Box<[u8]>);
