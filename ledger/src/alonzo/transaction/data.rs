use crate::{
    allegra::{self, Script},
    alonzo::script::PlutusV1,
    shelley::{self, transaction::Metadatum},
};
use displaydoc::Display;
use thiserror::Error;
use tinycbor::{container::map, *};
use tinycbor_derive::{CborLen, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, CborLen)]
#[cbor(map, tag(259))]
pub struct Data<'a> {
    #[cbor(n(0), optional)]
    pub metadata: shelley::transaction::Data<'a>,
    #[cbor(n(1), optional)]
    pub native_scripts: Vec<Script<'a>>,
    #[cbor(n(2), optional)]
    pub plutus_scripts: Vec<&'a PlutusV1>,
}

#[derive(Debug, Display, Error)]
pub enum Error {
    /// while decoding shelley style metadata
    Shelley(
        #[from]
        container::Error<
            map::Error<primitive::Error, <Metadatum<'static> as Decode<'static>>::Error>,
        >,
    ),
    /// while decoding allegra style data
    Allegra(#[from] <allegra::transaction::data::codec::Codec<'static> as Decode<'static>>::Error),
    /// while decoding alonzo style data
    Alonzo(#[from] <codec::Codec<'static> as Decode<'static>>::Error),
}

impl<'a, 'b: 'a> Decode<'b> for Data<'a> {
    type Error = Error;

    fn decode(d: &mut Decoder<'b>) -> Result<Self, Self::Error> {
        match d.datatype() {
            Ok(Type::Map | Type::MapIndef) => Ok(Data {
                metadata: shelley::transaction::Data::decode(d)?,
                native_scripts: Vec::new(),
                plutus_scripts: Vec::new(),
            }),
            Ok(Type::Array | Type::ArrayIndef) => {
                let allegra::transaction::data::codec::Codec { metadata, scripts } =
                    allegra::transaction::data::codec::Codec::decode(d)?;
                Ok(Data {
                    metadata,
                    native_scripts: scripts,
                    plutus_scripts: Vec::new(),
                })
            }
            _ => {
                let codec::Codec {
                    metadata,
                    native_scripts,
                    plutus_scripts,
                } = codec::Codec::decode(d)?;
                Ok(Data {
                    metadata,
                    native_scripts,
                    plutus_scripts,
                })
            }
        }
    }
}

pub(crate) mod codec {
    use crate::{allegra::Script, alonzo::script::PlutusV1, shelley};
    use tinycbor_derive::Decode;

    #[derive(Decode)]
    #[cbor(map, tag(259))]
    pub struct Codec<'a> {
        #[cbor(n(0), optional)]
        pub metadata: shelley::transaction::Data<'a>,
        #[cbor(n(1), optional)]
        pub native_scripts: Vec<Script<'a>>,
        #[cbor(n(2), optional)]
        pub plutus_scripts: Vec<&'a PlutusV1>,
    }
}
