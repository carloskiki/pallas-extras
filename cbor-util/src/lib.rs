pub mod boxed_slice;
pub mod list_as_map;
pub mod set;

use decode::{BytesIter, StrIter};
pub use minicbor::*;

pub mod bool_as_u8 {
    use minicbor::{CborLen, Decoder, Encoder, decode as de, encode as en};

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

    pub fn cbor_len<C>(value: &bool, ctx: &mut C) -> usize {
        (*value as u8).cbor_len(ctx)
    }
}

pub mod bounded_bytes {
    use minicbor::{
        Decoder, Encoder,
        bytes::ByteSlice,
        decode as de, encode as en,
    };

    pub fn encode<C, W: en::Write>(
        value: &[u8],
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        if value.len() <= 64 {
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
        match d.datatype()? {
            minicbor::data::Type::Bytes => {
                let bytes: &ByteSlice = d.decode()?;
                if bytes.len() > 64 {
                    Err(de::Error::message("byte slice too long for bounded bytes"))
                } else {
                    Ok(Box::from(&**bytes))
                }
            },
            minicbor::data::Type::BytesIndef => {
                let mut bytes = Vec::with_capacity(64);
                for slice in  d.bytes_iter()? {
                    let slice = slice?;
                    if slice.len() > 64 {
                        return Err(de::Error::message("byte slice too long for bounded bytes"))
                    }
                    bytes.extend_from_slice(slice);
                }
                Ok(bytes.into_boxed_slice())
            },
            t => Err(de::Error::type_mismatch(t).at(d.position()))
        }
    }

    pub fn cbor_len<Ctx>(value: &[u8], ctx: &mut Ctx) -> usize {
        if value.len() <= 64 {
            minicbor::bytes::cbor_len(value, ctx)
        } else {
            2 + value
                .chunks(64)
                .map(|c| minicbor::bytes::cbor_len(c, ctx))
                .sum::<usize>()
        }
    }
}

/// For anything that implements `SignatureEncoding`.
pub mod signature {
    use minicbor::{CborLen, Decoder, Encoder, decode as de, encode as en};
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

    pub fn cbor_len<S, Ctx>(value: &S, ctx: &mut Ctx) -> usize
    where
        S: SignatureEncoding,
    {
        let len = value.encoded_len();
        len.cbor_len(ctx) + len
    }
}

/// Encode a type as a byte array that contains the CBOR encoding of the type with tag 24.
pub mod cbor_encoded {
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
}

pub mod url {
    use minicbor::{CborLen, Decoder, Encoder};

    pub fn encode<C, W: minicbor::encode::Write>(
        value: &str,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.str(value)?.ok()
    }

    pub fn decode<C>(d: &mut Decoder<'_>, _: &mut C) -> Result<Box<str>, minicbor::decode::Error> {
        let string = d.str()?;
        if string.len() > 128 {
            Err(minicbor::decode::Error::message("url too long").at(d.position()))
        } else {
            Ok(Box::from(string))
        }
    }

    pub fn cbor_len<C>(value: &str, c: &mut C) -> usize {
        value.cbor_len(c)
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

pub fn array_decode<'a, T, F: FnOnce(&mut Decoder<'a>) -> Result<T, minicbor::decode::Error>>(
    len: u64,
    f: F,
    d: &mut Decoder<'a>,
) -> Result<T, minicbor::decode::Error> {
    let arr_len = d.array()?;
    if arr_len.is_some_and(|l| l != len) {
        return Err(minicbor::decode::Error::message(format!(
            "expected array of length {}",
            len
        )));
    }
    let ret = f(d)?;

    if arr_len.is_none() {
        if d.datatype()? != minicbor::data::Type::Break {
            return Err(minicbor::decode::Error::message(format!(
                "expected array of length {}",
                len
            )));
        }
        d.skip()?;
    }

    Ok(ret)
}
