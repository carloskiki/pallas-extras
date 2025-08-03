//! Hard Fork Combinator
//!
//! Encode and decode data that uses the hard fork combinator format.

use minicbor::{decode as de, encode as en, CborLen, Decoder, Encoder};

pub fn encode<C, W: en::Write, T: en::Encode<C> + CborLen<C>>(
    value: (&T, ledger::protocol::Era),
    e: &mut Encoder<W>,
    ctx: &mut C,
) -> Result<(), en::Error<W::Error>> {
    e.array(2)?.encode(value.1)?;
    cbor_util::cbor_encoded::encode(value.0, e, ctx)?;
    Ok(())
}

pub fn decode<'a, T: for<'b> de::Decode<'b, Ctx>, Ctx>(
    d: &mut Decoder<'a>,
    ctx: &mut Ctx,
) -> Result<(T, ledger::protocol::Era), de::Error> {
    cbor_util::array_decode(2, |d| {
        let era = d.decode::<ledger::protocol::Era>()?;
        let value = T::decode(d, ctx)?;
        Ok((value, era))
    }, d)
}
