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

