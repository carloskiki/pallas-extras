use std::convert::Infallible;

use tinycbor::{
    CborLen, Decode, Encode, Encoder, Write,
    container::{self, bounded},
};
use zerocopy::{Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Immutable, Unaligned, IntoBytes, KnownLayout,
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

impl<'a> TryFrom<&'a [u8]> for &'a Name {
    type Error = bounded::Error<Infallible>;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() > 32 {
            return Err(bounded::Error::Surplus);
        }

        // SAFETY: `repr(C)` guarantees that `Name` has the same layout as `[u8]`
        unsafe { Ok(&*(value as *const [u8] as *const Name)) }
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
    type Error = container::Error<bounded::Error<Infallible>>;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        <&'a [u8]>::decode(d)?
            .try_into()
            .map_err(container::Error::Content)
    }
}
