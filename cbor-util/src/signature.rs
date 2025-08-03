//! For anything that implements `SignatureEncoding`.
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

