use super::Data;
use tinycbor::{container::bounded, *};
use tinycbor_derive::{CborLen, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Encode, CborLen)]
#[cbor(tag(102))]
pub struct Construct {
    pub tag: u64,
    pub value: Vec<Data>,
}

impl Decode<'_> for Construct {
    type Error = tag::Error<container::Error<bounded::Error<Error>>>;

    fn decode(d: &mut Decoder<'_>) -> Result<Self, Self::Error> {
        fn wrap(e: impl Into<Error>) -> tag::Error<container::Error<bounded::Error<Error>>> {
            tag::Error::Content(container::Error::Content(bounded::Error::Content(e.into())))
        }

        let tag = match d.next().ok_or(EndOfInput)? {
            Ok(Token::Tag(t)) => t,
            Err(container::Error::Malformed(e)) => return Err(e.into()),
            _ => {
                return Err(InvalidHeader.into());
            }
        };
        let (tag, value) = match tag {
            121..=127 => (tag - 121, <Vec<Data>>::decode(d).map_err(wrap)?),
            1280..=1400 => (tag - 1280 + 7, Decode::decode(d).map_err(wrap)?),
            102 => {
                let mut visitor = d
                    .array_visitor()
                    .map_err(|e| tag::Error::Content(e.into()))?;
                let tag: u64 = visitor
                    .visit()
                    .ok_or(tag::Error::Content(bounded::Error::Missing.into()))?
                    .map_err(wrap)?;
                let value: Vec<Data> = visitor
                    .visit()
                    .ok_or(tag::Error::Content(bounded::Error::Missing.into()))?
                    .map_err(wrap)?;
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
    #[error("failed to decode large tag")]
    Tag(#[from] primitive::Error),
    #[error("failed to decode construct value")]
    Value(#[from] container::Error<<Data as Decode<'static>>::Error>),
}
