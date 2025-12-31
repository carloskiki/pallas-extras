use super::Data;
use tinycbor::*;
use tinycbor_derive::{CborLen, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Encode, CborLen)]
#[cbor(tag(102))]
pub struct Construct {
    pub tag: u64,
    pub value: Vec<Data>,
}

impl Decode<'_> for Construct {
    type Error = tag::Error<collections::fixed::Error<Error>>;

    fn decode(d: &mut Decoder<'_>) -> Result<Self, Self::Error> {
        fn wrap(e: Error) -> tag::Error<collections::fixed::Error<Error>> {
            tag::Error::Inner(collections::fixed::Error::Collection(
                collections::Error::Element(e),
            ))
        }

        let tag = match d
            .next()
            .ok_or(tag::Error::Malformed(primitive::Error::EndOfInput(
                EndOfInput,
            )))? {
            Ok(Token::Tag(t)) => t,
            Ok(_) | Err(string::Error::Utf8(_)) => {
                return Err(tag::Error::Malformed(primitive::Error::InvalidHeader(
                    InvalidHeader,
                )));
            }
            Err(string::Error::Malformed(e)) => return Err(tag::Error::Malformed(e)),
        };
        let (tag, value) = match tag {
            121..=127 => (
                tag - 121,
                <Vec<Data>>::decode(d).map_err(|e| wrap(Error::Value(e)))?,
            ),
            1280..=1400 => (
                tag - 1280 + 7,
                Decode::decode(d).map_err(|e| wrap(Error::Value(e)))?,
            ),
            102 => {
                let mut visitor = d.array_visitor().map_err(|e| {
                    tag::Error::Inner(collections::fixed::Error::Collection(
                        collections::Error::Malformed(e),
                    ))
                })?;
                let tag: u64 = visitor
                    .visit()
                    .ok_or(tag::Error::Inner(collections::fixed::Error::Missing))?
                    .map_err(|e| wrap(Error::from(e)))?;
                let value: Vec<Data> = visitor
                    .visit()
                    .ok_or(tag::Error::Inner(collections::fixed::Error::Missing))?
                    .map_err(|e| wrap(Error::from(e)))?;
                (tag, value)
            }
            _ => {
                return Err(tag::Error::InvalidTag);
            }
        };

        Ok(Self { tag, value })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("failed to decode large tag: {0}")]
    Tag(#[from] primitive::Error),
    #[error("failed to decode construct value: {0}")]
    Value(#[from] collections::Error<super::Error>),
}
