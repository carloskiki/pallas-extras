use tinycbor::{Any, CborLen, Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Attributes<'a>(Any<'a>);

impl core::ops::Deref for Attributes<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::convert::AsRef<[u8]> for Attributes<'_> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a, 'b> Decode<'b> for Attributes<'a>
where
    'b: 'a,
{
    type Error = tinycbor::string::Error;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        if !matches!(
            d.datatype()?,
            tinycbor::Type::Map | tinycbor::Type::MapIndef,
        ) {
            return Err(tinycbor::InvalidHeader.into());
        }

        Ok(Attributes(tinycbor::Any::decode(d)?))
    }
}

impl Encode for Attributes<'_> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        e.0.write_all(&self.0)
    }
}

impl CborLen for Attributes<'_> {
    fn cbor_len(&self) -> usize {
        self.0.len()
    }
}
