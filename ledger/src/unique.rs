use displaydoc::Display;
use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash},
    ops::Deref,
};
use thiserror::Error;
use tinycbor::{
    CborLen, Decode, Encode,
    container::{self, map},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display, Error)]
pub enum Error<E> {
    /// duplicate elements found in set
    Duplicate,
    /// inner error
    Content(#[from] E),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unique<T, const STRICT: bool>(T);

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
        decode_dedup_by_key(|| visitor.visit(), |(k, _)| k)
            .map(|(_, v)| Self(v))
            .map_err(container::Error::Content)
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
        let (removed, v) = decode_dedup_by_key(|| visitor.visit(), |(k, _)| k)
            .map_err(|e| container::Error::Content(Error::Content(e)))?;
        if removed {
            return Err(container::Error::Content(Error::Duplicate));
        }
        Ok(Self(v))
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

fn decode_dedup_by_key<T, E, K: Hash + Eq>(
    mut value: impl FnMut() -> Option<Result<T, E>>,
    key: impl Fn(&T) -> &K,
) -> Result<(bool, Vec<T>), E> {
    use hashbrown::{HashTable, hash_table::Entry};

    let mut set = HashTable::new();
    let random_state = RandomState::new();
    let make_hash = |s: &K| random_state.hash_one(s);
    let mut v = Vec::new();
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

    Ok((removed, v))
}

fn dedup<T: Hash + Eq>(v: &mut Vec<T>) -> bool {
    use hashbrown::{HashTable, hash_table::Entry};

    let mut set = HashTable::new();
    let random_state = RandomState::new();
    let make_hash = |s: &T| random_state.hash_one(s);

    let len = v.len();
    let mut del = 0;
    for i in 0..len {
        let current = &v[i];
        let hash = make_hash(current);
        match set.entry(hash, |&j| &v[j] == current, |&j| make_hash(&v[j])) {
            Entry::Occupied(_) => {
                del += 1;
            }
            Entry::Vacant(entry) => {
                if del > 0 {
                    v.swap(i - del, i);
                }
                entry.insert(i - del);
            }
        }
    }
    if del > 0 {
        v.truncate(len - del);
        true
    } else {
        false
    }
}

pub(crate) mod codec {
    use tinycbor::{EndOfInput, InvalidHeader, tag};

    use super::*;

    pub struct List<T>(Unique<Vec<T>, false>);

    impl<T> From<List<T>> for Unique<Vec<T>, false> {
        fn from(value: List<T>) -> Self {
            value.0
        }
    }

    impl<'a, T: Decode<'a> + Hash + Eq> Decode<'a> for List<T> {
        type Error = tinycbor::container::Error<<T as Decode<'a>>::Error>;

        fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
            let mut visitor = d.array_visitor()?;
            decode_dedup_by_key(|| visitor.visit(), |x| x)
                .map(|(_, v)| Self(Unique(v)))
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
            match d.peekable().next() {
                Some(Ok(tinycbor::Token::Tag(258))) => {
                    let _ = d.next().expect("tag was peeked");
                }
                Some(Ok(tinycbor::Token::Tag(_))) => return Err(InvalidHeader.into()),
                Some(Ok(tinycbor::Token::Array(_) | tinycbor::Token::BeginArray)) => {}
                Some(Err(container::Error::Malformed(e))) => return Err(tag::Error::Malformed(e)),
                Some(_) => return Err(InvalidHeader.into()),
                None => return Err(EndOfInput.into()),
            }
            
            let mut visitor = d.array_visitor()?;
            decode_dedup_by_key(|| visitor.visit(), |x| x)
                .map(|(_, v)| Self(Unique(v)))
                .map_err(|e| tag::Error::Content(container::Error::Content(e)))
        }
    }
}
