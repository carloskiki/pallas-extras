use std::convert::Infallible;

use tinycbor::{
    CborLen, Decode, Encode, Encoder, Write,
    container::{self, bounded},
    string,
};
use zerocopy::{Immutable, IntoBytes, KnownLayout, Unaligned};

/// Url.
///
/// This wraps `str`, ensuring its length is bounded by `MAX`.
///
/// In pre-[`conway`](crate::conway) eras `MAX == 64`, otherwise `MAX == 128`.
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Immutable, Unaligned, IntoBytes, KnownLayout,
)]
#[repr(C)]
pub struct Url<const MAX: usize>(str);

impl<const MAX: usize> AsRef<str> for Url<MAX> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<const MAX: usize> AsMut<str> for Url<MAX> {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl<'a, const MAX: usize> TryFrom<&'a str> for &'a Url<MAX> {
    type Error = bounded::Error<Infallible>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.len() > MAX {
            return Err(bounded::Error::Surplus);
        }
        unsafe { Ok(&*(value as *const str as *const Url<_>)) }
    }
}

impl<const MAX: usize> Encode for Url<MAX> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        self.0.encode(e)
    }
}

impl<const MAX: usize> CborLen for Url<MAX> {
    fn cbor_len(&self) -> usize {
        self.0.cbor_len()
    }
}

impl<'a, 'b: 'a, const MAX: usize> Decode<'b> for &'a Url<MAX> {
    type Error = container::Error<bounded::Error<string::InvalidUtf8>>;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        Ok(
            <&Url<_>>::try_from(<&str>::decode(d).map_err(|e| e.map(bounded::Error::Content))?)
                .map_err(|e| e.map(|e| match e {}))?,
        )
    }
}
