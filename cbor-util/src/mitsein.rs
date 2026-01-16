use mitsein::EmptyError;
use tinycbor::{CborLen, Decode, Encode, num::nonzero};

#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct NonEmpty<T>(pub mitsein::NonEmpty<T>);

impl<T> From<NonEmpty<T>> for mitsein::NonEmpty<T> {
    fn from(wrapper: NonEmpty<T>) -> Self {
        wrapper.0
    }
}

impl<'a, T> From<&'a mitsein::NonEmpty<T>> for &'a NonEmpty<T> {
    fn from(value: &'a mitsein::NonEmpty<T>) -> Self {
        use ref_cast::RefCast;
        NonEmpty::ref_cast(value)
    }
}

impl<'a, T> Decode<'a> for NonEmpty<T>
where
    T: Decode<'a>,
    mitsein::NonEmpty<T>: TryFrom<T, Error = EmptyError<T>>,
{
    type Error = nonzero::Error<T::Error>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        let value = T::decode(d).map_err(nonzero::Error::Value)?;
        mitsein::NonEmpty::try_from(value)
            .map(NonEmpty)
            .map_err(|_| nonzero::Error::Zero)
    }
}

impl<T> Encode for NonEmpty<T>
where
    T: Encode,
{
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        self.0.as_ref().encode(e)
    }
}

impl<T> CborLen for NonEmpty<T>
where
    T: CborLen,
{
    fn cbor_len(&self) -> usize {
        self.0.as_ref().cbor_len()
    }
}
