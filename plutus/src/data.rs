//! Implementation of the `Data` constant used by plutus.

use std::str::FromStr;
use tinycbor::*;

use crate::lex;

pub mod construct;
pub use construct::Construct;

/// The `Data` constant used by plutus.
// TODO: move this to the ledger, as more and more types from the ledger are being used here, so
// the ledger should host this type.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Data {
    Map(Vec<(Data, Data)>),
    List(Vec<Data>),
    Bytes(Vec<u8>),
    Integer(rug::Integer),
    Construct(Construct),
}

impl Default for Data {
    fn default() -> Self {
        Data::Integer(Default::default())
    }
}

impl CborLen for Data {
    fn cbor_len(&self) -> usize {
        match self {
            Data::Map(items) => items.cbor_len(),
            Data::List(datas) => datas.cbor_len(),
            Data::Bytes(items) => {
                <Vec<u8> as AsRef<cbor_util::tinycbor::BoundedBytes>>::as_ref(items).cbor_len()
            }
            Data::Integer(big_int) => big_int.as_ref().cbor_len(),
            Data::Construct(construct) => construct.cbor_len(),
        }
    }
}

impl Encode for Data {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        match self {
            Data::Map(items) => items.encode(e),
            Data::List(items) => items.encode(e),
            Data::Bytes(bytes) => {
                <Vec<u8> as AsRef<cbor_util::tinycbor::BoundedBytes>>::as_ref(bytes).encode(e)
            }
            Data::Integer(big_int) => big_int.as_ref().encode(e),
            Data::Construct(construct) => construct.encode(e),
        }
    }
}

impl Decode<'_> for Data {
    type Error = Error;

    fn decode(d: &mut Decoder<'_>) -> Result<Self, Self::Error> {
        match d.datatype()? {
            Type::Int => cbor_util::tinycbor::BigInt::decode(d)
                .map(|b| Self::Integer(b.0))
                .map_err(Error::Integer),
            Type::Bytes | Type::BytesIndef => {
                Ok(Self::Bytes(cbor_util::tinycbor::BoundedBytes::decode(d)?.0))
            }
            Type::Array | Type::ArrayIndef => Ok(Self::List(Decode::decode(d).map_err(Box::new)?)),
            Type::Map | Type::MapIndef => Ok(Self::Map(Decode::decode(d).map_err(Box::new)?)),
            Type::Tag => {
                let pre = *d;
                match Construct::decode(d) {
                    Ok(c) => return Ok(Self::Construct(c)),
                    Err(tag::Error::InvalidTag) => {}
                    Err(e) => return Err(Error::Construct(Box::new(e))),
                }
                *d = pre;
                
                cbor_util::tinycbor::BigInt::decode(d)
                    .map(|b| Self::Integer(b.0))
                    .map_err(Error::Integer)
            }
            _ => Err(Error::Header(primitive::Error::InvalidHeader(
                InvalidHeader,
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
enum Error {
    #[error("while decoding header: {0}")]
    Header(#[from] primitive::Error),
    #[error("while decoding map: {0}")]
    Map(#[from] Box<collections::Error<collections::map::Error<Error, Error>>>),
    #[error("while decoding list: {0}")]
    List(#[from] Box<collections::Error<Error>>),
    #[error("while decoding bytes: {0}")]
    Bytes(#[from] cbor_util::tinycbor::bounded_bytes::Error),
    #[error("while decoding integer: {0}")]
    Integer(#[from] cbor_util::tinycbor::Error),
    #[error("while decoding construct: {0}")]
    Construct(#[from] Box<<Construct as Decode<'static>>::Error>),
}

pub(crate) fn parse_data(s: &str) -> Option<(Data, &str)> {
    let (ty, data_str) = s
        .split_once(char::is_whitespace)
        .map(|(a, b)| (a, b.trim_start()))
        .unwrap_or((s, ""));
    let (word_str, mut rest) = data_str
        .find(',')
        .map(|pos| (data_str[..pos].trim_end(), &data_str[pos..]))
        .unwrap_or((data_str.trim_end(), ""));
    let data = match ty {
        "B" => {
            let hex = word_str.strip_prefix("#")?;
            let bytes = const_hex::decode(hex).ok()?;
            Data::Bytes(bytes)
        }
        "I" => {
            let int = rug::Integer::from_str_radix(word_str, 10).ok()?;
            Data::Integer(int)
        }
        "List" => {
            let (data, list_rest) = data_list(data_str)?;
            rest = list_rest;
            Data::List(data)
        }
        "Map" => {
            let (mut items_str, map_rest) = lex::group::<b'[', b']'>(data_str)?;
            rest = map_rest.strip_prefix(',').unwrap_or(map_rest).trim_start();
            let mut items = Vec::new();
            while !items_str.is_empty() {
                let (pair, rest) = lex::group::<b'(', b')'>(items_str)?;
                items_str = rest.strip_prefix(',').unwrap_or(rest).trim_start();
                let (key, rest) = parse_data(pair)?;
                let (value, "") = parse_data(rest.strip_prefix(',')?.trim_start())? else {
                    return None;
                };

                items.push((key, value));
            }

            Data::Map(items)
        }
        "Constr" => {
            let (tag_str, tag_rest) = data_str.split_once(char::is_whitespace)?;
            let tag = u64::from_str(tag_str).ok()?;
            let (value, constr_rest) = data_list(tag_rest)?;
            rest = constr_rest;
            Data::Construct(Construct { tag, value })
        }
        _ => return None,
    };

    Some((data, rest))
}

fn data_list(s: &str) -> Option<(Vec<Data>, &str)> {
    let (mut items_str, rest) = lex::group::<b'[', b']'>(s)?;
    let mut items = Vec::new();
    while !items_str.is_empty() {
        let (item, mut list_rest) = parse_data(items_str)?;
        items.push(item);
        if let Some(rest) = list_rest.strip_prefix(',') {
            list_rest = rest.trim_start();
        } else if !list_rest.is_empty() {
            return None;
        }
        items_str = list_rest;
    }

    Some((items, rest))
}
