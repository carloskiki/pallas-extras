use std::collections::HashSet;

use displaydoc::Display;
use thiserror::Error;
use tinycbor::{
    CborLen, Decode, Encode, EndOfInput, InvalidHeader, container,
    tag::{self, Tagged},
};

pub struct Set<T, const STRICT: bool>(pub HashSet<T>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Error)]
pub enum Error<E> {
    /// duplicate elements found in set
    Duplicate,
    /// inner error
    Content(#[from] E),
}

fn tag<E>(d: &mut tinycbor::Decoder<'_>) -> Result<(), tag::Error<container::Error<E>>> {
    match d.peekable().next() {
        Some(Ok(tinycbor::Token::Tag(258))) => {
            let _ = d.next().expect("tag was peeked");
            Ok(())
        }
        Some(Ok(tinycbor::Token::Tag(_))) => Err(InvalidHeader.into()),
        Some(Ok(tinycbor::Token::Array(_) | tinycbor::Token::BeginArray)) => Ok(()),
        Some(Ok(_)) => Err(InvalidHeader.into()),
        Some(Err(e)) => Err(match e {
            tinycbor::string::Error::Malformed(error) => tag::Error::Malformed(error),
            tinycbor::string::Error::Utf8 => InvalidHeader.into(),
        }),
        None => Err(EndOfInput.into()),
    }
}

impl<'a, T> Decode<'a> for Set<T, true>
where
    T: Decode<'a> + Eq + std::hash::Hash,
{
    type Error = tag::Error<container::Error<Error<T::Error>>>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        tag(d)?;
        
        let mut set = HashSet::new();
        let mut visitor = d.array_visitor()?;
        while let Some(elem) = visitor.visit() {
            if !set.insert(
                elem.map_err(|e| {
                    tag::Error::Content(container::Error::Content(Error::Content(e)))
                })?,
            ) {
                todo!()
            }
        }
        Ok(Set(set))
    }
}

impl<'a, T> Decode<'a> for Set<T, false>
where
    T: Decode<'a> + Eq + std::hash::Hash,
{
    type Error = tag::Error<container::Error<T::Error>>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        tag(d)?;
        
        let mut set = HashSet::new();
        let mut visitor = d.array_visitor()?;
        while let Some(elem) = visitor.visit() {
            set.insert(
                elem.map_err(|e| tag::Error::Content(container::Error::Content(e)))?,
            );
        }
        Ok(Set(set))
    }
}

impl<T, const STRICT: bool> Encode for Set<T, STRICT>
where
    T: Encode,
{
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        Tagged::<_, 258>(&self.0).encode(e)
    }
}

impl<T, const STRICT: bool> CborLen for Set<T, STRICT>
where
    T: CborLen,
{
    fn cbor_len(&self) -> usize {
        Tagged::<_, 258>(&self.0).cbor_len()
    }
}
