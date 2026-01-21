use crate::conway::{Asset, asset, transaction::Coin};
use std::num::NonZeroU64;
use tinycbor::{
    CborLen, Decode, Encode,
    collections::{self, fixed},
};
use tinycbor_derive::Decode;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value<'a> {
    Lovelace(Coin),
    Other {
        lovelace: Coin,
        assets: Asset<'a, NonZeroU64>,
    },
}

#[derive(Decode)]
struct Inner<'a> {
    lovelace: Coin,
    #[cbor(decode_with = "asset::Codec<'_, NonZeroU64>")]
    assets: Asset<'a, NonZeroU64>,
}

impl Encode for Value<'_> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        match self {
            Value::Lovelace(lovelace) => lovelace.encode(e),
            Value::Other { lovelace, assets } => {
                e.array(2)?;
                lovelace.encode(e)?;
                <asset::Codec<NonZeroU64> as ref_cast::RefCast>::ref_cast(assets).encode(e)
            }
        }
    }
}

impl<'a, 'b: 'a> Decode<'b> for Value<'a> {
    type Error = collections::Error<fixed::Error<Error>>;

    // TODO: check if there was a story of pruning empty bundles here, and for which eras.
    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        match d.datatype()? {
            tinycbor::Type::Int => return Ok(Value::Lovelace(u64::decode(d)?)),
            _ => {}
        }

        let Inner { lovelace, assets } = Inner::decode(d)?;
        Ok(Value::Other { lovelace, assets })
    }
}

impl CborLen for Value<'_> {
    fn cbor_len(&self) -> usize {
        match self {
            Value::Lovelace(lovelace) => lovelace.cbor_len(),
            Value::Other { lovelace, assets } => {
                2.cbor_len()
                    + lovelace.cbor_len()
                    + <asset::Codec<NonZeroU64> as ref_cast::RefCast>::ref_cast(assets).cbor_len()
            }
        }
    }
}
