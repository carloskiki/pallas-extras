use minicbor::{CborLen, Decode, Encode};

use crate::crypto::Blake2b224Digest;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(transparent)]
pub struct Asset<T>(
    #[cbor(
        with = "cbor_util::list_as_map::key_bytes",
        bound = "T: Encode<Ctx> + for<'a> Decode<'a, Ctx>"
    )]
    pub Box<[(Blake2b224Digest, Bundle<T>)]>,
);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(transparent)]
pub struct Bundle<T>(
    #[cbor(
        with = "cbor_util::list_as_map",
        bound = "T: Encode<Ctx> + for<'a> Decode<'a, Ctx>"
    )]
    pub Box<[(Name, T)]>,
);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, CborLen)]
#[cbor(transparent)]
pub struct Name(#[cbor(with = "minicbor::bytes")] pub Box<[u8]>);

impl<C> Decode<'_, C> for Name {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let bytes = d.bytes()?;
        if bytes.len() > 32 {
            return Err(minicbor::decode::Error::message("asset name is too long").at(d.position()))
        }
        Ok(Name(bytes.into()))
    }
}
