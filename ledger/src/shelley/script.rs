use crate::crypto::Blake2b224Digest;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(recursive)]
pub enum Script<'a> {
    #[n(0)]
    Vkey(&'a Blake2b224Digest),
    #[n(1)]
    All(Box<[Script<'a>]>),
    #[n(2)]
    Any(Box<[Script<'a>]>),
    #[n(3)]
    NofK(u64, Box<[Script<'a>]>),
}
