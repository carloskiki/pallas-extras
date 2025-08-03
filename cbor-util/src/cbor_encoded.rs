//! Encode a type as a byte array that contains the CBOR encoding of the type with tag 24.
use minicbor::{CborLen, Decode, Decoder, Encode, Encoder, decode as de, encode as en};
use std::fmt::Debug;

use crate::bytes_iter_collect;

pub fn encode<C, W: en::Write, T: en::Encode<C> + CborLen<C>>(
    value: &T,
    e: &mut Encoder<W>,
    ctx: &mut C,
) -> Result<(), en::Error<W::Error>> {
    e.tag(minicbor::data::Tag::new(24))?;
    e.bytes_len(value.cbor_len(ctx) as u64)?;
    e.encode_with(value, ctx)?.ok()
}

pub fn decode<'a, T: for<'b> de::Decode<'b, Ctx> + Debug, Ctx>(
    d: &mut Decoder<'a>,
    ctx: &mut Ctx,
) -> Result<T, de::Error> {
    let tag = d.tag()?;
    if tag != minicbor::data::Tag::new(24) {
        return Err(de::Error::tag_mismatch(tag));
    }

    let store;
    let bytes;
    match d.datatype()? {
        minicbor::data::Type::Bytes => {
            bytes = d.bytes()?;
        }
        minicbor::data::Type::BytesIndef => {
            store = bytes_iter_collect(d.bytes_iter()?)?;
            bytes = &store;
        }
        t => return Err(de::Error::type_mismatch(t).at(d.position())),
    }

    let mut inner_decoder = Decoder::new(bytes);
    inner_decoder.decode_with(ctx)
}

pub fn nil<'a, T: Decode<'a, ()>>() -> Option<T> {
    T::nil()
}

pub fn is_nil<T: Encode<()>>(v: &T) -> bool {
    v.is_nil()
}

pub fn cbor_len<C, T: CborLen<C>>(v: &T, ctx: &mut C) -> usize {
    let len = v.cbor_len(ctx);
    24.cbor_len(ctx) + len + len.cbor_len(ctx)
}

