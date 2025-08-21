use minicbor::{CborLen, Decode, Encode};

use crate::{crypto::Blake2b224Digest, protocol::Era};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(transparent)]
pub struct Asset<T>(
    #[cbor(
        with = "cbor_util::list_as_map::key_bytes",
        bound = "T: Encode<Ctx> + for<'a> Decode<'a, Ctx>"
    )]
    pub Box<[(Blake2b224Digest, Bundle<T>)]>,
);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, CborLen)]
#[cbor(transparent)]
pub struct Bundle<T>(#[cbor(with = "cbor_util::list_as_map")] pub Box<[(Name, T)]>);

impl<T: Encode<Era> + for<'a> Decode<'a, Era>> Encode<Era> for Bundle<T> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        era: &mut Era,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        if *era < Era::Mary {
            return Err(minicbor::encode::Error::message(
                "assets are not supported in eras before Mary",
            ));
        }
        cbor_util::list_as_map::encode(&self.0, e, era)
    }
}

impl<T: for<'a> Decode<'a, Era>> Decode<'_, Era> for Bundle<T> {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        era: &mut Era,
    ) -> Result<Self, minicbor::decode::Error> {
        if *era < Era::Mary {
            return Err(minicbor::decode::Error::message(
                "assets are not supported in eras before Mary",
            )
            .at(d.position()));
        }
        cbor_util::list_as_map::decode(d, era).map(Bundle)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, CborLen)]
#[cbor(transparent)]
pub struct Name(#[cbor(with = "minicbor::bytes")] pub Box<[u8]>);

impl<C> Decode<'_, C> for Name {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let bytes = d.bytes()?;
        if bytes.len() > 32 {
            return Err(minicbor::decode::Error::message("asset name is too long").at(d.position()));
        }
        Ok(Name(bytes.into()))
    }
}
