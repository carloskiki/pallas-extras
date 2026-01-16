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
pub struct Url(str);

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsMut<str> for Url {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl<'a> TryFrom<&'a str> for &'a Url {
    type Error = TooLong;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.len() > 64 {
            return Err(TooLong);
        }
        Ok(Url::try_ref_from_bytes(value.as_bytes()).expect("valid str"))
    }
}

impl Encode for Url {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        self.0.encode(e)
    }
}

impl CborLen for Url {
    fn cbor_len(&self) -> usize {
        self.0.cbor_len()
    }
}

impl<'a, 'b: 'a> Decode<'b> for &'a Url {
    type Error = Error;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        Ok(<&Url>::try_from(<&str>::decode(d)?)?)
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
