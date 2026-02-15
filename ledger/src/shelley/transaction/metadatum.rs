use displaydoc::Display;
use thiserror::Error;
use tinycbor::{
    CborLen, Decode, Encode, Type,
    container::{self, map},
    primitive, string,
};

pub type Label = u64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Metadatum<'a> {
    Integer(tinycbor::num::Int),
    Bytes(&'a [u8]),
    Text(&'a str),
    List(Vec<Metadatum<'a>>),
    Map(Vec<(Metadatum<'a>, Metadatum<'a>)>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error, Display)]
pub enum Error {
    /// while decoding `Integer`
    Integer(#[source] primitive::Error),
    /// while decoding `Bytes`
    Bytes(#[from] primitive::Error),
    /// while decoding `Text`
    Text(#[from] container::Error<string::InvalidUtf8>),
    /// while decoding `List`
    List(#[from] container::Error<Box<Error>>),
    /// while decoding `Map`
    Map(#[from] container::Error<Box<map::Error<Error, Error>>>),
}

impl<'a, 'b: 'a> Decode<'b> for Metadatum<'a> {
    type Error = Error;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        match d.datatype() {
            Ok(Type::Int) => Decode::decode(d)
                .map(Metadatum::Integer)
                .map_err(Error::Integer),
            Ok(Type::Bytes) => Decode::decode(d)
                .map(Metadatum::Bytes)
                .map_err(Error::Bytes),
            Ok(Type::String) => Decode::decode(d).map(Metadatum::Text).map_err(Error::Text),
            Ok(Type::Array | Type::ArrayIndef) => Decode::decode(d)
                .map(Metadatum::List)
                .map_err(|e| Error::List(e.map(Box::new))),
            Ok(Type::Map | Type::MapIndef) => Decode::decode(d)
                .map(Metadatum::Map)
                .map_err(|e| Error::Map(e.map(Box::new))),
            Err(e) => Err(Error::Bytes(e.into())),
            _ => Err(Error::Bytes(primitive::Error::InvalidHeader)),
        }
    }
}

impl Encode for Metadatum<'_> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        match self {
            Metadatum::Integer(i) => i.encode(e),
            Metadatum::Bytes(b) => b.encode(e),
            Metadatum::Text(s) => s.encode(e),
            Metadatum::List(l) => l.encode(e),
            Metadatum::Map(m) => m.encode(e),
        }
    }
}

impl CborLen for Metadatum<'_> {
    fn cbor_len(&self) -> usize {
        match self {
            Metadatum::Integer(i) => i.cbor_len(),
            Metadatum::Bytes(b) => b.cbor_len(),
            Metadatum::Text(s) => s.cbor_len(),
            Metadatum::List(l) => l.cbor_len(),
            Metadatum::Map(m) => m.cbor_len(),
        }
    }
}
