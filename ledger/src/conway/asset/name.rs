use displaydoc::Display;
use thiserror::Error;
use tinycbor::{CborLen, Decode, Encode, Encoder, Write, collections};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name<'a>(&'a [u8]);

impl Encode for Name<'_> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        self.0.encode(e)
    }
}

impl CborLen for Name<'_> {
    fn cbor_len(&self) -> usize {
        self.0.cbor_len()
    }
}

impl<'a, 'b: 'a> Decode<'b> for Name<'a> {
    type Error = collections::Error<TooLong>;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        let bytes: &'b [u8] = Decode::decode(d)?;
        if bytes.len() > 32 {
            return Err(collections::Error::Element(TooLong));
        }
        Ok(Name(bytes))
    }
}

/// the name is longer than 32 bytes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Error)]
pub struct TooLong;
