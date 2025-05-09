use minicbor::{CborLen, Decode, Encode};

use crate::crypto::Blake2b224Digest;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(transparent)]
pub struct Asset<T>(
    #[cbor(
        with = "cbor_util::list_as_map::key_bytes",
        bound = "T: Encode<Ctx> + for<'a> Decode<'a, Ctx>"
    )]
    pub Box<[(Blake2b224Digest, Bundle<T>)]>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(transparent)]
pub struct Bundle<T>(
    #[cbor(
        with = "cbor_util::list_as_map::key_bytes",
        bound = "T: Encode<Ctx> + for<'a> Decode<'a, Ctx>"
    )]
    pub Box<[([u8; 32], T)]>,
);
