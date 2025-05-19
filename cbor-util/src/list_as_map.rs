use minicbor::{CborLen, Decoder, Encoder, decode as de, encode as en};

#[allow(clippy::borrowed_box)]
pub fn encode<C, W: en::Write, T: en::Encode<C>, U: en::Encode<C>>(
    value: &Box<[(T, U)]>,
    e: &mut Encoder<W>,
    ctx: &mut C,
) -> Result<(), en::Error<W::Error>> {
    e.map(value.len() as u64)?;
    for (v, u) in value.iter() {
        e.encode_with(v, ctx)?;
        e.encode_with(u, ctx)?;
    }
    Ok(())
}

pub fn decode<'a, T: de::Decode<'a, Ctx>, U: de::Decode<'a, Ctx>, Ctx>(
    d: &mut Decoder<'a>,
    ctx: &mut Ctx,
) -> Result<Box<[(T, U)]>, de::Error> {
    d.map_iter_with(ctx)?.collect::<Result<Box<[(T, U)]>, _>>()
}

pub fn nil<K, V>() -> Option<Box<[(K, V)]>> {
    Some(Default::default())
}

#[allow(clippy::borrowed_box)]
pub fn is_nil<K, V>(v: &Box<[(K, V)]>) -> bool {
    v.is_empty()
}

#[allow(clippy::borrowed_box)]
pub fn cbor_len<Ctx, K: CborLen<Ctx>, V: CborLen<Ctx>>(
    val: &Box<[(K, V)]>,
    ctx: &mut Ctx,
) -> usize {
    val.len().cbor_len(ctx)
        + val
            .iter()
            .map(|(k, v)| k.cbor_len(ctx) + v.cbor_len(ctx))
            .sum::<usize>()
}

pub mod key_bytes {
    use minicbor::{
        CborLen, Decoder, Encoder,
        bytes::{self, CborLenBytes},
        decode as de, encode as en,
    };

    #[allow(clippy::borrowed_box)]
    pub fn encode<C, W: en::Write, T: bytes::EncodeBytes<C>, U: en::Encode<C>>(
        value: &Box<[(T, U)]>,
        e: &mut Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        e.map(value.len() as u64)?;
        for (v, u) in value.iter() {
            v.encode_bytes(e, ctx)?;
            e.encode_with(u, ctx)?;
        }
        Ok(())
    }

    pub fn decode<'a, T: bytes::DecodeBytes<'a, Ctx>, U: de::Decode<'a, Ctx>, Ctx>(
        d: &mut Decoder<'a>,
        ctx: &mut Ctx,
    ) -> Result<Box<[(T, U)]>, de::Error> {
        let mut map_len = d.map()?;
        let mut container = Vec::with_capacity(map_len.unwrap_or(0) as usize);
        while map_len.is_none_or(|l| l != 0) && d.datatype()? != minicbor::data::Type::Break {
            let key = minicbor::bytes::decode(d, ctx)?;
            let value = d.decode_with(ctx)?;
            container.push((key, value));
            map_len = map_len.and_then(|l| l.checked_sub(1));
        }
        if map_len.is_none() {
            let ty = d.datatype()?;
            if ty != minicbor::data::Type::Break {
                return Err(minicbor::decode::Error::type_mismatch(ty).at(d.position()));
            }
            d.skip()?;
        }

        Ok(container.into_boxed_slice())
    }

    pub fn nil<K, V>() -> Option<Box<[(K, V)]>> {
        Some(Default::default())
    }

    #[allow(clippy::borrowed_box)]
    pub fn is_nil<K, V>(v: &Box<[(K, V)]>) -> bool {
        v.is_empty()
    }

    #[allow(clippy::borrowed_box)]
    pub fn cbor_len<C, K: CborLenBytes<C>, V: CborLen<C>>(
        value: &Box<[(K, V)]>,
        ctx: &mut C,
    ) -> usize {
        value.len().cbor_len(ctx)
            + value
                .iter()
                .map(|(key, val)| key.cbor_len(ctx) + val.cbor_len(ctx))
                .sum::<usize>()
    }
}
