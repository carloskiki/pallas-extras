use crate::{
    allegra::Script,
    shelley::{self, transaction::Metadatum},
};
use displaydoc::Display;
use thiserror::Error;
use tinycbor::{container::map, *};
use tinycbor_derive::{CborLen, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, CborLen)]
pub struct Data<'a> {
    pub metadata: shelley::transaction::Data<'a>,
    pub scripts: Vec<Script<'a>>,
}

#[derive(Debug, Display, Error)]
pub enum Error {
    /// while decoding shelley style data
    Shelley(
        #[from]
        container::Error<
            map::Error<primitive::Error, <Metadatum<'static> as Decode<'static>>::Error>,
        >,
    ),
    /// while decoding allegra style data
    Allegra(#[from] <codec::Codec<'static> as Decode<'static>>::Error),
}

impl<'a, 'b: 'a> Decode<'b> for Data<'a> {
    type Error = Error;

    fn decode(d: &mut Decoder<'b>) -> Result<Self, Self::Error> {
        match d.datatype() {
            Ok(Type::Map | Type::MapIndef) => Ok(Data {
                metadata: shelley::transaction::Data::decode(d)?,
                scripts: Vec::new(),
            }),
            _ => {
                let codec::Codec { metadata, scripts } = codec::Codec::decode(d)?;
                Ok(Data { metadata, scripts })
            }
        }
    }
}

pub(crate) mod codec {
    use tinycbor_derive::{Decode};

    use crate::shelley::transaction::Data;

    #[derive(Decode)]
    pub struct Codec<'a> {
        pub metadata: Data<'a>,
        pub scripts: Vec<super::Script<'a>>,
    }

}
