use crate::{
    mary::asset::{self, Asset},
    shelley::transaction::Coin,
};
use tinycbor::{
    CborLen, Decode, Encode,
    container::{self, bounded},
};
use tinycbor_derive::Decode;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value<'a> {
    Lovelace(Coin),
    Other {
        lovelace: Coin,
        assets: Asset<'a, u64>,
    },
}

#[derive(Decode)]
struct Inner<'a> {
    lovelace: Coin,
    #[cbor(decode_with = "asset::Codec<'_, u64>")]
    assets: Asset<'a, u64>,
}

impl Encode for Value<'_> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        match self {
            Value::Lovelace(lovelace) => lovelace.encode(e),
            Value::Other { lovelace, assets } => {
                e.array(2)?;
                lovelace.encode(e)?;
                <asset::Codec<u64> as ref_cast::RefCast>::ref_cast(assets).encode(e)
            }
        }
    }
}

impl<'a, 'b: 'a> Decode<'b> for Value<'a> {
    type Error = container::Error<bounded::Error<Error>>;

    // TODO: check if there was a story of pruning empty bundles here, and for which eras.
    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        if d.datatype()? == tinycbor::Type::Int {
            return Ok(Value::Lovelace(u64::decode(d)?));
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
                    + <asset::Codec<u64> as ref_cast::RefCast>::ref_cast(assets).cbor_len()
            }
        }
    }
}
