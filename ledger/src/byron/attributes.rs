use tinycbor::{CborLen, Decode, Encode};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Attributes(Vec<u8>);

impl core::ops::Deref for Attributes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::convert::AsRef<[u8]> for Attributes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Decode<'_> for Attributes {
    type Error = tinycbor::string::Error;

    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        if !matches!(
            d.datatype()?,
            tinycbor::Type::Map | tinycbor::Type::MapIndef,
        ) {
            return Err(tinycbor::string::Error::Malformed(
                tinycbor::primitive::Error::InvalidHeader(tinycbor::InvalidHeader),
            ));
        }
        
        let any = tinycbor::Any::decode(d)?;
        let bytes: &[u8] = any.as_ref();
        Ok(Attributes(bytes.to_vec()))
    }
}

impl Encode for Attributes {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        e.0.write_all(&self.0)
    }
}

impl CborLen for Attributes {
    fn cbor_len(&self) -> usize {
        self.0.len()
    }
}
