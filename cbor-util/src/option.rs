use tinycbor::{
    CborLen, Decode, Encode,
    collections::{self, fixed},
    tag,
};

#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct Array<T, const TAGGED: bool>(pub Option<T>);

impl<T, const TAGGED: bool> From<Array<T, TAGGED>> for Option<T> {
    fn from(wrapper: Array<T, TAGGED>) -> Self {
        wrapper.0
    }
}

impl<'a, T, const TAGGED: bool> From<&'a Option<T>> for &'a Array<T, TAGGED> {
    fn from(value: &'a Option<T>) -> Self {
        use ref_cast::RefCast;
        Array::ref_cast(value)
    }
}

impl<T> Encode for Array<T, false>
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

impl<T> Encode for Array<T, true>
where
    T: Encode,
{
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        match &self.0 {
            Some(v) => {
                e.array(2)?;
                1u64.encode(e)?;
                v.encode(e)
            }
            None => {
                e.array(1)?;
                0u64.encode(e)
            }
        }
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for Array<T, false> {
    type Error = collections::Error<fixed::Error<T::Error>>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        let mut visitor = d.array_visitor()?;
        let ret = visitor
            .visit()
            .transpose()
            .map_err(|e| collections::Error::Element(fixed::Error::Inner(e)))?;
        if visitor.remaining() != Some(0) {
            return Err(collections::Error::Element(fixed::Error::Surplus));
        }
        Ok(Array(ret))
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for Array<T, true> {
    type Error = collections::Error<fixed::Error<tag::Error<T::Error>>>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        let mut visitor = d.array_visitor()?;
        let tag: u64 = visitor
            .visit()
            .ok_or(collections::Error::Element(fixed::Error::Missing))?
            .map_err(|e| {
                collections::Error::Element(fixed::Error::Inner(tag::Error::Malformed(e)))
            })?;
        let ret = match tag {
            0 => None,
            1 => Some(
                visitor
                    .visit()
                    .ok_or(collections::Error::Element(fixed::Error::Missing))?
                    .map_err(|e| {
                        collections::Error::Element(fixed::Error::Inner(tag::Error::Inner(e)))
                    })?,
            ),
            _ => {
                return Err(collections::Error::Element(fixed::Error::Inner(
                    tag::Error::InvalidTag,
                )));
            }
        };
        if visitor.remaining() != Some(0) {
            return Err(collections::Error::Element(fixed::Error::Surplus));
        }
        Ok(Array(ret))
    }
}

impl<T: CborLen> CborLen for Array<T, false> {
    fn cbor_len(&self) -> usize {
        match &self.0 {
            Some(v) => 1 + v.cbor_len(),
            None => 1,
        }
    }
}

impl<T: CborLen> CborLen for Array<T, true> {
    fn cbor_len(&self) -> usize {
        match &self.0 {
            Some(v) => 1 + 1 + v.cbor_len(),
            None => 1 + 1,
        }
    }
}
