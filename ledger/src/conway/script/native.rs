use tinycbor_derive::{CborLen, Decode, Encode};

use crate::{crypto::Blake2b224Digest, slot};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(recursive)]
pub enum Script<'a> {
    #[n(0)]
    Vkey(&'a Blake2b224Digest),
    #[n(1)]
    All(Vec<Script<'a>>),
    #[n(2)]
    Any(Vec<Script<'a>>),
    #[n(3)]
    NofK(u64, Vec<Script<'a>>),
    #[n(4)]
    InvalidBefore(slot::Number),
    #[n(5)]
    InvalidHereafter(slot::Number),
}
