use minicbor::{CborLen, Decoder, Encoder, decode as de, encode as en};

pub fn encode<C, W: en::Write, T: en::Encode<C>>(
    value: &[T],
    e: &mut Encoder<W>,
    ctx: &mut C,
) -> Result<(), en::Error<W::Error>> {
    e.encode_with(value, ctx)?.ok()
}

pub fn decode<'a, T: de::Decode<'a, Ctx>, Ctx>(
    d: &mut Decoder<'a>,
    ctx: &mut Ctx,
) -> Result<Box<[T]>, de::Error> {
    let x = d.array_iter_with(ctx)?.collect::<Result<Box<[_]>, _>>()?;
    
    Ok(x)
}

pub fn nil<T>() -> Option<Box<[T]>> {
    Some(Vec::new().into_boxed_slice())
}

pub fn is_nil<T>(v: &[T]) -> bool {
    v.is_empty()
}

pub fn cbor_len<Ctx, T: CborLen<Ctx>>(val: &[T], ctx: &mut Ctx) -> usize {
    val.cbor_len(ctx)
}

pub mod bytes {
    use minicbor::{
        CborLen, Decoder, Encoder,
        bytes::{self, CborLenBytes},
        decode as de, encode as en,
    };

    pub fn encode<C, W: en::Write, T: bytes::EncodeBytes<C>>(
        value: &[T],
        e: &mut Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        e.array(value.len() as u64)?;
        for v in value.iter() {
            v.encode_bytes(e, ctx)?;
        }
        Ok(())
    }

    pub fn decode<'a, T: bytes::DecodeBytes<'a, Ctx>, Ctx>(
        d: &mut Decoder<'a>,
        ctx: &mut Ctx,
    ) -> Result<Box<[T]>, de::Error> {
        let mut len = d.array()?;
        let mut container = Vec::with_capacity(len.unwrap_or(0) as usize);
        while len.is_none_or(|l| l != 0) && d.datatype()? != minicbor::data::Type::Break {
            let value = bytes::decode(d, ctx)?;
            container.push(value);
            len = len.and_then(|l| l.checked_sub(1));
        }
        if len.is_none() {
            let ty = d.datatype()?;
            if ty != minicbor::data::Type::Break {
                return Err(de::Error::type_mismatch(ty).at(d.position()));
            }
            d.skip()?;
        }

        Ok(container.into_boxed_slice())
    }

    pub fn cbor_len<Ctx, T: CborLenBytes<Ctx>>(val: &[T], ctx: &mut Ctx) -> usize {
        val.len().cbor_len(ctx) + val.iter().map(|v| v.cbor_len(ctx)).sum::<usize>()
    }

    pub use super::{is_nil, nil};
}
