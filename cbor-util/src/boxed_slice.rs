use std::ops::Deref;

use minicbor::{CborLen, Decoder, Encoder, decode as de, encode as en};

#[allow(clippy::borrowed_box)]
pub fn encode<C, W: en::Write, T: en::Encode<C>>(
    value: &Box<[T]>,
    e: &mut Encoder<W>,
    ctx: &mut C,
) -> Result<(), en::Error<W::Error>> {
    e.array(value.len() as u64)?;
    for v in value.iter() {
        e.encode_with(v, ctx)?;
    }
    Ok(())
}

pub fn decode<'a, T: de::Decode<'a, Ctx>, Ctx>(
    d: &mut Decoder<'a>,
    ctx: &mut Ctx,
) -> Result<Box<[T]>, de::Error> {
    let v: Vec<T> = d.decode_with(ctx)?;
    Ok(v.into_boxed_slice())
}

pub fn nil<T>() -> Option<Box<[T]>> {
    Some(Vec::new().into_boxed_slice())
}

#[allow(clippy::borrowed_box)]
pub fn is_nil<T>(v: &Box<[T]>) -> bool {
    v.is_empty()
}

#[allow(clippy::borrowed_box)]
pub fn cbor_len<Ctx, T: CborLen<Ctx>>(val: &Box<[T]>, ctx: &mut Ctx) -> usize {
    val.deref().cbor_len(ctx)
}

pub mod bytes {
    use minicbor::{
        CborLen, Decoder, Encoder,
        bytes::{self, CborLenBytes},
        decode as de, encode as en,
    };

    #[allow(clippy::borrowed_box)]
    pub fn encode<C, W: en::Write, T: bytes::EncodeBytes<C>>(
        value: &Box<[T]>,
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
                return Err(de::Error::type_mismatch(ty));
            }
            d.skip()?;
        }

        Ok(container.into_boxed_slice())
    }

    #[allow(clippy::borrowed_box)]
    pub fn cbor_len<Ctx, T: CborLenBytes<Ctx>>(val: &Box<[T]>, ctx: &mut Ctx) -> usize {
        val.len().cbor_len(ctx) + val.iter().map(|v| v.cbor_len(ctx)).sum::<usize>()
    }

    pub use super::{is_nil, nil};
}
