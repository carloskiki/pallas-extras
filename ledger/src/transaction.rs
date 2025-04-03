use cbor_util::{array_iter_collect, bytes_iter_collect, map_iter_collect, str_iter_collect};
use minicbor::{Decode, Encode};

use crate::{
    crypto::Blake2b256Digest,
    script::{native, plutus},
};

use super::{
    address::shelley::{Address, StakeAddress},
    certificate, protocol, witness,
};

#[derive(Debug, Clone, PartialEq, Eq, Encode)]
pub struct Transaction {
    #[n(0)]
    pub body: Body,
    #[n(1)]
    pub witness_set: witness::Set,
    #[n(2)]
    pub valid_scripts: bool,
    #[n(3)]
    pub data: Data,
}

impl<C> Decode<'_, C> for Transaction {
    fn decode(d: &mut minicbor::Decoder<'_>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        let len = d.array()?;
        match len {
            Some(3 | 4) | None => {
                let tx = Transaction {
                    body: d.decode()?,
                    witness_set: d.decode()?,
                    valid_scripts: if d.datatype()? == minicbor::data::Type::Bool {
                        d.decode()?
                    } else {
                        true
                    },
                    data: d.decode()?,
                };
                if len.is_none() {
                    if d.datatype()? != minicbor::data::Type::Break {
                        return Err(minicbor::decode::Error::message("Invalid array length"));
                    };
                    d.skip()?;
                }
                Ok(tx)
            }
            t => Err(minicbor::decode::Error::message("Invalid array length")),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode)]
#[cbor(map, tag(259))]
pub struct Data {
    #[cbor(n(0), with = "cbor_util::list_as_map", has_nil)]
    metadata: Box<[(u64, Metadatum)]>,
    #[cbor(n(1), with = "cbor_util::boxed_slice", has_nil)]
    auxiliary_scripts: Box<[native::Script]>,
    #[cbor(n(2), with = "cbor_util::boxed_slice", has_nil)]
    plutus_v1: Box<[plutus::Script]>,
    #[cbor(n(3), with = "cbor_util::boxed_slice", has_nil)]
    plutus_v2: Box<[plutus::Script]>,
}

impl<C> Decode<'_, C> for Data {
    fn decode(d: &mut minicbor::Decoder<'_>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        use minicbor::data::Type;
        match d.datatype()? {
            Type::Tag => {
                // Alonzo era data structure
                #[derive(Decode)]
                #[cbor(map, tag(259))]
                pub struct DataAlonzo {
                    #[cbor(n(0), with = "cbor_util::list_as_map", has_nil)]
                    metadata: Box<[(u64, Metadatum)]>,
                    #[cbor(n(1), with = "cbor_util::boxed_slice", has_nil)]
                    auxiliary_scripts: Box<[native::Script]>,
                    #[cbor(n(2), with = "cbor_util::boxed_slice", has_nil)]
                    plutus_v1: Box<[plutus::Script]>,
                    #[cbor(n(3), with = "cbor_util::boxed_slice", has_nil)]
                    plutus_v2: Box<[plutus::Script]>,
                }
                
                let DataAlonzo {
                    metadata,
                    auxiliary_scripts,
                    plutus_v1,
                    plutus_v2,
                } = d.decode()?;
                
                Ok(Data {
                    metadata,
                    auxiliary_scripts,
                    plutus_v1,
                    plutus_v2,
                })
            },
            Type::Array | Type::ArrayIndef => {
                // Multi asset era data structure
                #[derive(Decode)]
                struct DataMA {
                    #[cbor(n(0), with = "cbor_util::list_as_map", has_nil)]
                    metadata: Box<[(u64, Metadatum)]>,
                    #[cbor(n(1), with = "cbor_util::boxed_slice", has_nil)]
                    auxiliary_scripts: Box<[native::Script]>,
                }
                let DataMA {
                    metadata,
                    auxiliary_scripts,
                } = d.decode()?;
                
                Ok(Data {
                    metadata,
                    auxiliary_scripts,
                    ..Default::default()
                })
            }
            Type::Map | Type::MapIndef => {
                Ok(Data {
                    metadata: cbor_util::list_as_map::decode(d, ctx)?,
                    ..Default::default()
                })
            }
            t => Err(minicbor::decode::Error::type_mismatch(t)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Metadatum {
    Integer(i64),
    Bytes(Box<[u8]>),
    Text(Box<str>),
    Array(Box<[Metadatum]>),
    Map(Box<[(Metadatum, Metadatum)]>),
}

impl<C> Encode<C> for Metadatum {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Metadatum::Integer(i) => e.i64(*i)?.ok(),
            Metadatum::Bytes(b) => e.bytes(b)?.ok(),
            Metadatum::Text(s) => e.str(s)?.ok(),
            Metadatum::Array(a) => {
                e.array(a.len() as u64)?;
                for item in a {
                    item.encode(e, ctx)?;
                }
                Ok(())
            }
            Metadatum::Map(m) => {
                e.map(m.len() as u64)?;
                for (k, v) in m {
                    k.encode(e, ctx)?;
                    v.encode(e, ctx)?;
                }
                Ok(())
            }
        }
    }
}

impl<C> Decode<'_, C> for Metadatum {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        use minicbor::data::Type;
        match d.datatype()? {
            Type::U8 | Type::U16 | Type::U32 | Type::U64 => Ok(Metadatum::Integer(d.u64()? as i64)),
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => Ok(Metadatum::Integer(d.i64()?)),
            Type::Bytes | Type::BytesIndef => Ok(Metadatum::Bytes(bytes_iter_collect(d.bytes_iter()?)?)),
            Type::String | Type::StringIndef => Ok(Metadatum::Text(str_iter_collect(d.str_iter()?)?)),
            Type::Array | Type::ArrayIndef => Ok(Metadatum::Array(array_iter_collect(d.array_iter()?)?)),
            Type::Map | Type::MapIndef => Ok(Metadatum::Map(map_iter_collect(d.map_iter()?)?)),
            t => Err(minicbor::decode::Error::type_mismatch(t)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Body {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub fee: u64,
    pub ttl: Option<u64>,
    pub certificates: Vec<certificate::Certificate>,
    pub withdrawals: Vec<(StakeAddress<false>, u64)>,
    pub update: Option<protocol::Update>,
    pub metadata_hash: Option<Blake2b256Digest>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Input {
    pub id: Blake2b256Digest,
    pub index: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Output {
    pub address: Address<false>,
    pub amount: u64,
}
