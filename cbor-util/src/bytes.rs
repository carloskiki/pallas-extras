use std::convert::Infallible;

use tinycbor::{
    CborLen, Decode, Encode, Encoder, Write,
    collections::{self, fixed},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct Bytes<'a, T>(pub &'a T);

impl<'a, T> Bytes<'a, T> {
    pub fn into(self) -> &'a T {
        self.0
    }
}

impl<'a, 'b, T> From<&'b &'a T> for &'b Bytes<'a, T> {
    fn from(value: &'b &'a T) -> Self {
        use ref_cast::RefCast;
        Bytes::ref_cast(value)
    }
}

impl<T: IntoBytes + Immutable> Encode for Bytes<'_, T> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        self.0.as_bytes().encode(e)
    }
}

impl<T: IntoBytes + Immutable> CborLen for Bytes<'_, T> {
    fn cbor_len(&self) -> usize {
        self.0.as_bytes().cbor_len()
    }
}

impl<'a, 'b: 'a, T: FromBytes + KnownLayout + Immutable + Unaligned> Decode<'b> for Bytes<'a, T> {
    type Error = fixed::Error<Infallible>;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        let bytes: &[u8] = Decode::decode(d)
            .map_err(|e| fixed::Error::Collection(collections::Error::Malformed(e)))?;

        T::ref_from_bytes(bytes)
            .map_err(|e| {
                if zerocopy::SizeError::from(e).into_src().len() > core::mem::size_of::<T>() {
                    fixed::Error::Surplus
                } else {
                    fixed::Error::Missing
                }
            })
            .map(|v| Bytes(v))
    }
}
