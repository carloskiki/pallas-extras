use std::convert::Infallible;

use crate::{Address, crypto::Blake2b256Digest, mary::transaction::Value};
use tinycbor::{*, container::bounded};
use thiserror::Error;
use displaydoc::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Output<'a> {
    pub address: Address<'a>,
    pub value: Value<'a>,
    pub datum_hash: Option<&'a Blake2b256Digest>,
}

/// error while decoding type `Output`
#[derive(Debug, Error, Display)]
#[prefix_enum_doc_attributes]
pub enum Error {
    /// in field `address`
    Address(#[from] <Address<'static> as Decode<'static>>::Error),
    /// in field `value`
    Value(#[from] <Value<'static> as Decode<'static>>::Error),
    /// in field `datum_hash`
    DatumHash(#[from] container::Error<bounded::Error<Infallible>>),
}

impl Encode for Output<'_> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        e.array(2 + self.datum_hash.is_some() as usize)?;
        self.address.encode(e)?;
        self.value.encode(e)?;
        if let Some(datum_hash) = &self.datum_hash {
            datum_hash.encode(e)?;
        }
        Ok(())
    }
}

impl CborLen for Output<'_> {
    fn cbor_len(&self) -> usize {
        1 + self.address.cbor_len()
            + self.value.cbor_len()
            + self.datum_hash.as_ref().map_or(0, |dh| dh.cbor_len())
    }
}

impl<'a, 'b: 'a> Decode<'b> for Output<'a> {
    type Error = container::Error<bounded::Error<Error>>;

    fn decode(d: &mut Decoder<'b>) -> Result<Self, Self::Error> {
        fn wrap(e: impl Into<Error>) -> bounded::Error<Error> {
            bounded::Error::Content(e.into())
        }
        
        let mut visitor = d.array_visitor()?;
        let address = visitor.visit().ok_or(bounded::Error::Missing)?.map_err(wrap)?;
        let value = visitor.visit().ok_or(bounded::Error::Missing)?.map_err(wrap)?;
        let datum_hash = visitor.visit().map(|opt| opt.map_err(wrap)).transpose()?;
        if visitor.remaining() != Some(0) {
            return Err(bounded::Error::Surplus.into());
        }

        Ok(Self {
            address,
            value,
            datum_hash,
        })
    }
}
