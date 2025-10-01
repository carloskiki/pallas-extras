use std::collections::HashSet;
use std::hash::Hash;

const TAG: Tag = Tag::new(258);

use minicbor::{CborLen, Decode, Decoder, Encoder, data::Tag, decode as de, encode as en};

pub fn encode<C, W: en::Write, T: en::Encode<C>>(
    value: &[T],
    e: &mut Encoder<W>,
    ctx: &mut C,
) -> Result<(), en::Error<W::Error>> {
    e.tag(TAG)?;
    e.encode_with(value, ctx)?.ok()
}

pub fn decode<Ctx, T: for<'a> Decode<'a, Ctx> + Hash + Eq>(
    d: &mut Decoder<'_>,
    ctx: &mut Ctx,
) -> Result<Box<[T]>, de::Error> {
    if d.datatype()? == minicbor::data::Type::Tag {
        let tag = d.tag()?;
        if tag != TAG {
            return Err(de::Error::tag_mismatch(tag).at(d.position()));
        }
        let v: Vec<T> = d.decode_with(ctx)?;
        let mut unique_check = HashSet::new();
        
        if !v.iter().all(move |x| unique_check.insert(x)) {
            // TODO: Figure out if this is also the case in mainnet, and 
            // return Err(de::Error::message("set is not unique").at(d.position()));
        } 
        Ok(v.into_boxed_slice())
    } else {
        let v: HashSet<T> = d.decode_with(ctx)?;
        Ok(FromIterator::from_iter(v))
    }
}

pub fn cbor_len<Ctx, T: CborLen<Ctx>>(val: &[T], ctx: &mut Ctx) -> usize {
    val.cbor_len(ctx) + TAG.cbor_len(ctx)
}

pub use crate::boxed_slice::{is_nil, nil};

pub mod bytes {
    use std::{collections::HashSet, hash::Hash};

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
        e.tag(TAG)?;
        e.array(value.len() as u64)?;
        for v in value.iter() {
            v.encode_bytes(e, ctx)?;
        }
        Ok(())
    }

    pub fn decode<'a, T: bytes::DecodeBytes<'a, Ctx> + Eq + Hash, Ctx>(
        d: &mut Decoder<'a>,
        ctx: &mut Ctx,
    ) -> Result<Box<[T]>, de::Error> {
        let has_tag = d.datatype()? == minicbor::data::Type::Tag;
        if has_tag {
            let tag = d.tag()?;
            if tag != TAG {
                return Err(de::Error::tag_mismatch(tag).at(d.position()));
            }
        }

        let mut len = d.array()?;
        let mut container = HashSet::with_capacity(len.unwrap_or(0) as usize);
        while len.is_none_or(|l| l != 0) && d.datatype()? != minicbor::data::Type::Break {
            let value = bytes::decode(d, ctx)?;
            if !container.insert(value) && has_tag {
                // TODO: Figure out this stuff
                // return Err(de::Error::message("set is not unique").at(d.position()));
            }
            len = len.and_then(|l| l.checked_sub(1));
        }
        if len.is_none() {
            let ty = d.datatype()?;
            if ty != minicbor::data::Type::Break {
                return Err(de::Error::type_mismatch(ty).at(d.position()));
            }
            d.skip()?;
        }

        Ok(FromIterator::from_iter(container))
    }

    pub fn cbor_len<Ctx, T: CborLenBytes<Ctx>>(val: &[T], ctx: &mut Ctx) -> usize {
        val.len().cbor_len(ctx)
            + val.iter().map(|v| v.cbor_len(ctx)).sum::<usize>()
            + TAG.cbor_len(ctx)
    }

    use super::TAG;
    pub use super::{is_nil, nil};
}
