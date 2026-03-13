use displaydoc::Display;
use mitsein::NonEmpty;
use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash},
    ops::Deref,
};
use thiserror::Error;
use tinycbor::{
    CborLen, Decode, Encode,
    container::{self, map},
    num::nonzero,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Error)]
pub enum Error<E> {
    /// duplicate elements found in set
    Duplicate,
    /// in unique content
    Content(#[from] E),
}

/// Ensures uniqueness of the elements in a `Vec` at deserialization time, maintaining insertion order.
///
/// the `STRICT` generic parameter determines whether construction errors or deduplicates
/// (maintaining the first instance) when encountering duplicates.
///
/// For `Vec<(K, V)>`, this ensures uniqueness of `K`.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unique<T, const STRICT: bool>(pub(crate) T);

impl<T, const STRICT: bool> Unique<T, STRICT> {
    pub fn inner(self) -> T {
        self.0
    }
}

impl<T, const STRICT: bool> Deref for Unique<T, STRICT> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, K, V> Decode<'a> for Unique<Vec<(K, V)>, false>
where
    K: Decode<'a> + Eq + std::hash::Hash,
    V: Decode<'a>,
    K:,
{
    type Error = container::Error<map::Error<<K as Decode<'a>>::Error, <V as Decode<'a>>::Error>>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        let mut visitor = d.map_visitor()?;
        let size_hint = visitor.remaining();
        decode_dedup_by_key(|| visitor.visit(), |(k, _)| k, size_hint)
            .map(|(_, u)| u)
            .map_err(container::Error::Content)
    }
}

impl<'a, K, V> Decode<'a> for Unique<NonEmpty<Vec<(K, V)>>, false>
where
    K: Decode<'a> + Eq + std::hash::Hash,
    V: Decode<'a>,
    K:,
{
    type Error = container::Error<
        nonzero::Error<map::Error<<K as Decode<'a>>::Error, <V as Decode<'a>>::Error>>,
    >;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        Unique::<Vec<(K, V)>, false>::decode(d)
            .map_err(|e| e.map(nonzero::Error::Value))
            .and_then(|Unique(a)| {
                NonEmpty::<Vec<_>>::try_from(a)
                    .map(Unique)
                    .map_err(|_| container::Error::Content(nonzero::Error::Zero))
            })
    }
}

impl<'a, K, V> Decode<'a> for Unique<Vec<(K, V)>, true>
where
    K: Decode<'a>,
    V: Decode<'a>,
    K: Eq + std::hash::Hash,
{
    type Error =
        container::Error<Error<map::Error<<K as Decode<'a>>::Error, <V as Decode<'a>>::Error>>>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        let mut visitor = d.map_visitor()?;
        let size_hint = visitor.remaining();
        let (removed, v) = decode_dedup_by_key(|| visitor.visit(), |(k, _)| k, size_hint)
            .map_err(|e| container::Error::Content(Error::Content(e)))?;
        if removed {
            return Err(container::Error::Content(Error::Duplicate));
        }
        Ok(v)
    }
}

impl<T: Encode, const STRICT: bool> Encode for Unique<T, STRICT> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        self.0.encode(e)
    }
}

impl<T: CborLen, const STRICT: bool> CborLen for Unique<T, STRICT> {
    fn cbor_len(&self) -> usize {
        self.0.cbor_len()
    }
}

pub(crate) fn decode_dedup_by_key<T, E, K: Hash + Eq, const STRICT: bool>(
    mut value: impl FnMut() -> Option<Result<T, E>>,
    key: impl Fn(&T) -> &K,
    size_hint: Option<usize>,
) -> Result<(bool, Unique<Vec<T>, STRICT>), E> {
    use hashbrown::{HashTable, hash_table::Entry};

    let random_state = RandomState::new();
    let make_hash = |s: &K| random_state.hash_one(s);
    let mut set = HashTable::with_capacity(size_hint.unwrap_or_default());
    let mut v = Vec::with_capacity(size_hint.unwrap_or_default());
    let mut removed = false;

    while let Some(x) = value() {
        let value = x?;
        let current = key(&value);
        let hash = make_hash(current);
        let i = v.len();

        match set.entry(hash, |&j| key(&v[j]) == current, |&j| make_hash(key(&v[j]))) {
            Entry::Occupied(_) => {
                removed = true;
            }
            Entry::Vacant(entry) => {
                entry.insert(i);
                v.push(value);
            }
        }
    }

    Ok((removed, Unique(v)))
}

// fn dedup<T: Hash + Eq>(v: &mut Vec<T>) -> bool {
//     use hashbrown::{HashTable, hash_table::Entry};
//
//     let mut set = HashTable::new();
//     let random_state = RandomState::new();
//     let make_hash = |s: &T| random_state.hash_one(s);
//
//     let len = v.len();
//     let mut del = 0;
//     for i in 0..len {
//         let current = &v[i];
//         let hash = make_hash(current);
//         match set.entry(hash, |&j| &v[j] == current, |&j| make_hash(&v[j])) {
//             Entry::Occupied(_) => {
//                 del += 1;
//             }
//             Entry::Vacant(entry) => {
//                 if del > 0 {
//                     v.swap(i - del, i);
//                 }
//                 entry.insert(i - del);
//             }
//         }
//     }
//     if del > 0 {
//         v.truncate(len - del);
//         true
//     } else {
//         false
//     }
// }

pub(crate) mod codec {
    use mitsein::vec1::Vec1;
    use tinycbor::{EndOfInput, InvalidHeader, num::nonzero, tag};

    use super::*;

    // TODO: Maybe this should be named `Untagged` and `Tagged` should be named `Set`?
    pub struct Set<T>(Unique<Vec<T>, false>);

    impl<T> From<Set<T>> for Unique<Vec<T>, false> {
        fn from(value: Set<T>) -> Self {
            value.0
        }
    }

    impl<'a, T: Decode<'a> + Hash + Eq> Decode<'a> for Set<T> {
        type Error = tinycbor::container::Error<<T as Decode<'a>>::Error>;

        fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
            let mut visitor = d.array_visitor()?;
            let size_hint = visitor.remaining();
            decode_dedup_by_key(|| visitor.visit(), |x| x, size_hint)
                .map(|(_, v)| Self(v))
                .map_err(tinycbor::container::Error::Content)
        }
    }

    pub struct Tagged<T>(Unique<Vec<T>, false>);

    impl<T> From<Tagged<T>> for Unique<Vec<T>, false> {
        fn from(value: Tagged<T>) -> Self {
            value.0
        }
    }

    impl<'a, T: Decode<'a> + Hash + Eq> Decode<'a> for Tagged<T> {
        type Error = tag::Error<container::Error<<T as Decode<'a>>::Error>>;

        fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
            let saved = *d;
            match d.next() {
                Some(Ok(tinycbor::Token::Tag(258))) => {}
                Some(Ok(tinycbor::Token::Array(_) | tinycbor::Token::BeginArray)) => {
                    *d = saved;
                }
                Some(Err(container::Error::Malformed(e))) => return Err(tag::Error::Malformed(e)),
                Some(Ok(tinycbor::Token::Tag(_))) => return Err(tag::Error::InvalidTag),
                Some(_) => return Err(InvalidHeader.into()),
                None => return Err(EndOfInput.into()),
            }

            Set::decode(d)
                .map(|Set(a)| Tagged(a))
                .map_err(tag::Error::Content)
        }
    }

    #[repr(transparent)]
    pub struct NonEmpty<T>(Unique<Vec1<T>, false>);

    impl<T> From<NonEmpty<T>> for Unique<Vec1<T>, false> {
        fn from(value: NonEmpty<T>) -> Self {
            value.0
        }
    }

    impl<T> From<NonEmpty<T>> for Unique<Vec<T>, false> {
        fn from(value: NonEmpty<T>) -> Self {
            Unique(value.0.0.into_vec())
        }
    }

    impl<'a, T> From<&'a Unique<Vec1<T>, false>> for &'a NonEmpty<T> {
        fn from(value: &'a Unique<Vec1<T>, false>) -> Self {
            // Safety: `NonEmpty` is `repr(transparent)`
            unsafe { &*(value as *const Unique<Vec1<T>, false> as *const NonEmpty<T>) }
        }
    }

    impl<'a, T: Decode<'a> + Hash + Eq> Decode<'a> for NonEmpty<T> {
        type Error = tag::Error<container::Error<nonzero::Error<<T as Decode<'a>>::Error>>>;

        fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
            Tagged::decode(d)
                .map_err(|e| e.map(|e| e.map(nonzero::Error::Value)))
                .and_then(|Tagged(Unique(s))| {
                    let Ok(non_empty) = Vec1::try_from(s) else {
                        return Err(tag::Error::Content(container::Error::Content(
                            nonzero::Error::Zero,
                        )));
                    };
                    Ok(NonEmpty(Unique(non_empty)))
                })
        }
    }

    impl<T> Encode for NonEmpty<T>
    where
        Vec<T>: Encode,
    {
        fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
            self.0.as_vec().encode(e)
        }
    }

    impl<T> CborLen for NonEmpty<T>
    where
        Vec<T>: CborLen,
    {
        fn cbor_len(&self) -> usize {
            self.0.as_vec().cbor_len()
        }
    }
}
