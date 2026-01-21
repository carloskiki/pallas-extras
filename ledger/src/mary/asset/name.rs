use crate::TooLong;
use displaydoc::Display;
use thiserror::Error;
use tinycbor::{CborLen, Decode, Encode, Encoder, Write, string};
use zerocopy::{FromZeros, Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    FromZeros,
    Immutable,
    Unaligned,
    IntoBytes,
    KnownLayout,
)]
#[repr(C)]
pub struct Name(pub [u8]);

impl AsRef<[u8]> for Name {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for Name {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl<'a> TryFrom<&'a str> for &'a Name {
    type Error = TooLong;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.len() > 64 {
            return Err(TooLong);
        }
        Ok(Name::try_ref_from_bytes(value.as_bytes()).expect("valid str"))
    }
}

impl Encode for Name {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        self.0.encode(e)
    }
}

impl CborLen for Name {
    fn cbor_len(&self) -> usize {
        self.0.cbor_len()
    }
}

impl<'a, 'b: 'a> Decode<'b> for &'a Name {
    type Error = Error;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        Ok(<&Name>::try_from(<&str>::decode(d)?)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Error)]
pub enum Error {
    /// the URL is Malformed
    Malformed(#[from] string::Error),
    /// the URL is too long
    TooLong,
}

impl From<TooLong> for Error {
    fn from(_: TooLong) -> Self {
        Error::TooLong
    }
}
