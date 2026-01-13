//! Implementation of the `Data` constant used by plutus.

use tinycbor::*;

pub mod construct;
pub use construct::Construct;

/// The `Data` constant used by plutus.
// TODO: Check if this can borrow bytes. There are potential problems with `plutus` crate.
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
            Data::Bytes(items) => <&cbor_util::BoundedBytes>::from(items).cbor_len(),
            Data::Integer(big_int) => <&cbor_util::BigInt>::from(big_int).cbor_len(),
            Data::Construct(construct) => construct.cbor_len(),
        }
    }
}

impl Encode for Data {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        match self {
            Data::Map(items) => items.encode(e),
            Data::List(items) => items.encode(e),
            Data::Bytes(bytes) => <&cbor_util::BoundedBytes>::from(bytes).encode(e),
            Data::Integer(big_int) => <&cbor_util::BigInt>::from(big_int).encode(e),
            Data::Construct(construct) => construct.encode(e),
        }
    }
}

impl Decode<'_> for Data {
    type Error = Error;

    fn decode(d: &mut Decoder<'_>) -> Result<Self, Self::Error> {
        match d.datatype().map_err(|e| Error::Header(From::from(e)))? {
            Type::Int => cbor_util::BigInt::decode(d)
                .map(|b| Self::Integer(b.0))
                .map_err(Error::Integer),
            Type::Bytes | Type::BytesIndef => {
                Ok(Self::Bytes(cbor_util::BoundedBytes::decode(d)?.0))
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

                cbor_util::BigInt::decode(d)
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
pub enum Error {
    #[error("while decoding header: {0}")]
    Header(#[from] primitive::Error),
    #[error("while decoding map: {0}")]
    Map(#[from] Box<collections::Error<collections::map::Error<Error, Error>>>),
    #[error("while decoding list: {0}")]
    List(#[from] Box<collections::Error<Error>>),
    #[error("while decoding bytes: {0}")]
    Bytes(#[from] cbor_util::bounded_bytes::Error),
    #[error("while decoding integer: {0}")]
    Integer(#[from] cbor_util::big_int::Error),
    #[error("while decoding construct: {0}")]
    Construct(#[from] Box<<Construct as Decode<'static>>::Error>),
}
