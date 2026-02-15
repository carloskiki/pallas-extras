use std::collections::HashSet;

use displaydoc::Display;
use thiserror::Error;
use tinycbor::{Decode, EndOfInput, InvalidHeader, container, tag};

// Implements the ordered set from the ledger spec.
pub struct Set<T, const STRICT: bool>(pub Vec<T>);

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
        Some(Err(container::Error::Malformed(e))) => Err(tag::Error::Malformed(e)),
        Some(_) => Err(InvalidHeader.into()),
        None => Err(EndOfInput.into()),
    }
}

impl<'a, T> Decode<'a> for Set<T, true>
where
    T: Decode<'a> + Eq + std::hash::Hash + Clone,
{
    type Error = tag::Error<container::Error<Error<T::Error>>>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        tag(d)?;

        let mut vec = Vec::new();
        let mut set = HashSet::new();
        let mut visitor = d.array_visitor()?;
        while let Some(elem) = visitor.visit() {
            let elem: T = elem
                .map_err(|e| tag::Error::Content(container::Error::Content(Error::Content(e))))?;
            if !set.insert(elem.clone()) {
                return Err(tag::Error::Content(container::Error::Content(
                    Error::Duplicate,
                )));
            }
            vec.push(elem);
        }
        Ok(Set(vec))
    }
}

impl<'a, T> Decode<'a> for Set<T, false>
where
    T: Decode<'a> + Eq + std::hash::Hash + Clone,
{
    type Error = tag::Error<container::Error<T::Error>>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        tag(d)?;

        let mut vec = Vec::new();
        let mut set = HashSet::new();
        let mut visitor = d.array_visitor()?;
        while let Some(elem) = visitor.visit() {
            let elem: T = elem.map_err(|e| tag::Error::Content(container::Error::Content(e)))?;
            set.insert(elem.clone());
            vec.push(elem);
        }
        Ok(Set(vec))
    }
}
