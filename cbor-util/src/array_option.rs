use tinycbor::{
    CborLen, Decode, Encode,
    collections::{self, fixed},
};

#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct ArrayOption<T>(pub Option<T>);

impl<T> From<ArrayOption<T>> for Option<T> {
    fn from(wrapper: ArrayOption<T>) -> Self {
        wrapper.0
    }
}

impl<'a, T> From<&'a Option<T>> for &'a ArrayOption<T> {
    fn from(value: &'a Option<T>) -> Self {
        use ref_cast::RefCast;
        ArrayOption::ref_cast(value)
    }
}

impl<T> Encode for ArrayOption<T>
where
    T: Encode,
{
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        match &self.0 {
            Some(v) => {
                e.array(1)?;
                v.encode(e)
            }
            None => e.array(0),
        }
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for ArrayOption<T> {
    type Error = fixed::Error<T::Error>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        let mut visitor = d.array_visitor().map_err(collections::Error::Malformed)?;
        let ret = visitor
            .visit()
            .transpose()
            .map_err(collections::Error::Element)?;
        if visitor.remaining() != Some(0) {
            return Err(fixed::Error::Surplus);
        }
        Ok(ArrayOption(ret))
    }
}

impl<T: CborLen> CborLen for ArrayOption<T> {
    fn cbor_len(&self) -> usize {
        match &self.0 {
            Some(v) => 1 + v.cbor_len(),
            None => 1,
        }
    }
}
