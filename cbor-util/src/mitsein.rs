use mitsein::{EmptyError, boxed1::BoxedSlice1};
use tinycbor::{CborLen, Decode, Encode, num::nonzero};

#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct NonEmpty<T>(pub BoxedSlice1<T>);

impl<T> From<NonEmpty<T>> for BoxedSlice1<T> {
    fn from(wrapper: NonEmpty<T>) -> Self {
        wrapper.0
    }
}

impl<'a, T> From<&'a BoxedSlice1<T>> for &'a NonEmpty<T> {
    fn from(value: &'a BoxedSlice1<T>) -> Self {
        use ref_cast::RefCast;
        NonEmpty::ref_cast(value)
    }
}

impl<'a, T> Decode<'a> for NonEmpty<T>
where
    Box<[T]>: Decode<'a>,
    BoxedSlice1<T>: TryFrom<Box<[T]>, Error = EmptyError<Box<[T]>>>,
{
    type Error = nonzero::Error<<Box<[T]> as Decode<'a>>::Error>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        let value = Box::<[T]>::decode(d).map_err(nonzero::Error::Value)?;
        BoxedSlice1::try_from(value)
            .map(NonEmpty)
            .map_err(|_| nonzero::Error::Zero)
    }
}

impl<T> Encode for NonEmpty<T>
where
    [T]: Encode,
{
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        self.0.as_slice().encode(e)
    }
}

impl<T> CborLen for NonEmpty<T>
where
    [T]: CborLen,
{
    fn cbor_len(&self) -> usize {
        self.0.as_slice().cbor_len()
    }
}
