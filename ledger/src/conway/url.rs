use tinycbor::{CborLen, Decode, Encode, Encoder, Write, container};

use crate::TooLong;

pub struct Url<'a>(&'a str);

impl AsRef<str> for Url<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl AsMut<str> for Url<'_> {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl TryFrom<&[u8]> for Url<'_> {
    type Error = TooLong;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() > 128 {
            return Err(TooLong);
        }
        Ok(Url(value))
    }
}

impl Encode for Url<'_> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        self.0.encode(e)
    }
}

impl CborLen for Url<'_> {
    fn cbor_len(&self) -> usize {
        self.0.cbor_len()
    }
}

impl<'a, 'b: 'a> Decode<'b> for Url<'a> {
    type Error = container::Error<TooLong>;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        let bytes: &'b [u8] = Decode::decode(d)?;
        Url::try_from(bytes).map_err(|e| container::Error::Content(e))
    }
}
