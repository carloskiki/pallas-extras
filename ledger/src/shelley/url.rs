use std::convert::Infallible;

use tinycbor::{
    CborLen, Decode, Encode, Encoder, Write,
    container::{self, bounded},
    string,
};
use zerocopy::{Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Immutable, Unaligned, IntoBytes, KnownLayout,
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
    type Error = bounded::Error<Infallible>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.len() > 64 {
            return Err(bounded::Error::Surplus);
        }
        unsafe { Ok(&*(value as *const str as *const Url)) }
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
    type Error = container::Error<bounded::Error<string::InvalidUtf8>>;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        Ok(
            <&Url>::try_from(<&str>::decode(d).map_err(|e| e.map(|e| bounded::Error::Content(e)))?)
                .map_err(|e| e.map(|e| match e {}))?,
        )
    }
}
