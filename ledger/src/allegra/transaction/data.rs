use crate::shelley::{Script, transaction::{Metadata, Metadatum}};
use displaydoc::Display;
use thiserror::Error;
use tinycbor::{
    Decode,
    container::{self, map},
    primitive,
};
use tinycbor_derive::{CborLen, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, CborLen)]
pub struct Data<'a> {
    metadata: Metadata<'a>,
    scripts: Vec<Script<'a>>,
}

#[derive(Debug, Display, Error)]
pub enum Error {
    /// while decoding standalone metadata
    Metadata(
        #[from]
        container::Error<
            map::Error<primitive::Error, <Metadatum<'static> as Decode<'static>>::Error>,
        >,
    ),
    /// while decoding `Data`
    Data(#[from] <array::Data<'static> as Decode<'static>>::Error),
}

impl<'a, 'b: 'a> Decode<'b> for Data<'a> {
    type Error = Error;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        if matches!(
            d.datatype(),
            Ok(tinycbor::Type::Map | tinycbor::Type::MapIndef)
        ) {
            Ok(Data {
                metadata: Metadata::decode(d)?,
                scripts: Vec::new(),
            })
        } else {
            let array::Data { metadata, scripts } = array::Data::decode(d)?;
            Ok(Data { metadata, scripts })
        }
    }
}

mod array {
    use crate::shelley::transaction::Metadata;
    
    use tinycbor_derive::{CborLen, Decode, Encode};
    #[derive(CborLen, Encode, Decode)]
    pub struct Data<'a> {
        pub metadata: Metadata<'a>,
        pub scripts: Vec<super::Script<'a>>,
    }
}
