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

