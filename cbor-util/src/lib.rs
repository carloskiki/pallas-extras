pub mod list_as_map;
pub mod boxed_slice;

use decode::{ArrayIter, BytesIter, MapIter, StrIter};
pub use minicbor::*;

pub mod bool_as_u8 {
    use minicbor::{Decoder, Encoder, decode as de, encode as en};

    pub fn encode<C, W: en::Write>(
        value: &bool,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        e.u8(*value as u8)?.ok()
    }

    pub fn decode<Ctx>(d: &mut Decoder<'_>, _: &mut Ctx) -> Result<bool, de::Error> {
        d.u8().map(|v| v != 0)
    }
}

pub mod boxed_bytes {
    use std::ops::Deref;

    use minicbor::{bytes::CborLenBytes, decode as de, encode as en, Decoder, Encoder};

    #[allow(clippy::borrowed_box)]
    pub fn encode<C, W: en::Write>(
        value: &Box<[u8]>,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        e.bytes(value)?.ok()
    }

    pub fn decode<Ctx>(
        d: &mut Decoder<'_>,
        ctx: &mut Ctx,
    ) -> Result<Box<[u8]>, de::Error> {
        let v: Vec<u8> = minicbor::bytes::decode(d, ctx)?;
        Ok(v.into_boxed_slice())
    }

    pub fn nil() -> Option<Box<[u8]>> {
        Some(Vec::new().into_boxed_slice())
    }

    #[allow(clippy::borrowed_box)]
    pub fn is_nil<>(v: &Box<[u8]>) -> bool {
        v.is_empty()
    }

    #[allow(clippy::borrowed_box)]
    pub fn cbor_len<Ctx>(val: &Box<[u8]>, ctx: &mut Ctx) -> usize {
        CborLenBytes::cbor_len(val.deref(), ctx)
    }
   
}

pub mod bounded_bytes {
    use minicbor::{Decoder, Encoder, bytes::ByteVec, decode as de, encode as en};

    #[allow(clippy::borrowed_box)]
    pub fn encode<C, W: en::Write>(
        value: &Box<[u8]>,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        if value.len() < 64 {
            e.bytes(value)?.ok()
        } else {
            e.begin_bytes()?;
            value
                .chunks(64)
                .try_for_each(|chunk| e.bytes(chunk)?.ok())?;
            e.end()?.ok()
        }
    }

    pub fn decode<Ctx>(d: &mut Decoder<'_>, _: &mut Ctx) -> Result<Box<[u8]>, de::Error> {
        // TODO: Here we do not check whether it respects the bounded bytes requirements.
        let v: ByteVec = d.decode()?;
        Ok(Vec::from(v).into_boxed_slice())
    }
}

/// For anything that implements `SignatureEncoding`.
pub mod signature {
    use minicbor::{Decoder, Encoder, decode as de, encode as en};
    use signature::SignatureEncoding;

    pub fn encode<C, S, W: en::Write>(
        value: &S,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), en::Error<W::Error>>
    where
        S: SignatureEncoding + TryInto<S::Repr, Error: core::error::Error + Send + Sync + 'static>,
    {
        let repr: S::Repr = value.clone().try_into().map_err(en::Error::custom)?;
        e.bytes(repr.as_ref())?.ok()
    }

    pub fn decode<S, Ctx>(d: &mut Decoder<'_>, _: &mut Ctx) -> Result<S, de::Error>
    where
        S: SignatureEncoding
            + for<'a> TryFrom<&'a [u8], Error: core::error::Error + Send + Sync + 'static>,
    {
        let bytes = d.bytes()?;
        S::try_from(bytes).map_err(de::Error::custom)
    }
}

/// Encode a type as a byte array that contains the CBOR encoding of the type with tag 24.
pub mod cbor_encoded {
    use minicbor::{decode as de, encode as en, CborLen, Decode, Decoder, Encode, Encoder};

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

    pub fn decode<'a, T: for<'b> de::Decode<'b, Ctx>, Ctx>(
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
            t => return Err(de::Error::type_mismatch(t)),
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
}

pub fn bytes_iter_collect(iter: BytesIter<'_, '_>) -> Result<Box<[u8]>, minicbor::decode::Error> {
    let mut bytes = Vec::with_capacity(iter.size_hint().0);
    for chunk in iter {
        bytes.extend_from_slice(chunk?);
    }
    Ok(bytes.into_boxed_slice())
}

pub fn str_iter_collect(iter: StrIter<'_, '_>) -> Result<Box<str>, minicbor::decode::Error> {
    let mut string = String::with_capacity(iter.size_hint().0);
    for chunk in iter {
        string.push_str(chunk?);
    }
    Ok(string.into_boxed_str())
}
