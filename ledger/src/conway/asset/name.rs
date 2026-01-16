use displaydoc::Display;
use thiserror::Error;
use tinycbor::{CborLen, Decode, Encode, Encoder, Write, collections};

use crate::TooLong;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name<'a>(&'a [u8]);

impl AsRef<[u8]> for Name<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl AsMut<[u8]> for Name<'_> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl TryFrom<&[u8]> for Name<'_> {
    type Error = TooLong;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() > 32 {
            return Err(TooLong);
        }
        Ok(Name(value))
    }
}

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
        Name::try_from(bytes).map_err(|e| collections::Error::Element(e))
    }
}
