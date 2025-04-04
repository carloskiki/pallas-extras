use cbor_util::{bytes_iter_collect, str_iter_collect};
use minicbor::{Decode, Encode};

use super::{
    address::shelley::{Address, StakeAddress},
    certificate, protocol, witness,
};
use crate::{
    asset::Asset,
    crypto::{Blake2b224Digest, Blake2b256Digest},
    script::{Script, native, plutus},
};

#[derive(Debug, Clone, PartialEq, Eq, Encode)]
pub struct Transaction<const MAINNET: bool> {
    #[n(0)]
    pub body: Body<MAINNET>,
    #[n(1)]
    pub witness_set: witness::Set,
    #[n(2)]
    pub valid_scripts: bool,
    #[n(3)]
    pub data: Data,
}

impl<const M: bool, C> Decode<'_, C> for Transaction<M> {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
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
            _ => Err(minicbor::decode::Error::message("Invalid array length")),
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
            }
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
            Type::Map | Type::MapIndef => Ok(Data {
                metadata: cbor_util::list_as_map::decode(d, ctx)?,
                ..Default::default()
            }),
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
            Type::Bytes | Type::BytesIndef => {
                Ok(Metadatum::Bytes(bytes_iter_collect(d.bytes_iter()?)?))
            }
            Type::String | Type::StringIndef => {
                Ok(Metadatum::Text(str_iter_collect(d.str_iter()?)?))
            }
            Type::Array | Type::ArrayIndef => Ok(Metadatum::Array(
                d.array_iter()?.collect::<Result<Box<[_]>, _>>()?,
            )),
            Type::Map | Type::MapIndef => Ok(Metadatum::Map(
                d.map_iter()?.collect::<Result<Box<[_]>, _>>()?,
            )),
            t => Err(minicbor::decode::Error::type_mismatch(t)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[cbor(map)]
pub struct Body<const MAINNET: bool> {
    #[cbor(n(0), with = "cbor_util::boxed_slice")]
    pub inputs: Box<[Input]>,
    #[cbor(n(1), with = "cbor_util::boxed_slice")]
    pub outputs: Box<[Output<MAINNET>]>,
    #[n(2)]
    pub fee: u64,
    #[n(3)]
    pub ttl: Option<u64>,
    #[cbor(n(4), with = "cbor_util::boxed_slice", has_nil)]
    pub certificates: Box<[certificate::Certificate<MAINNET>]>,
    #[cbor(n(5), with = "cbor_util::boxed_slice", has_nil)]
    pub withdrawals: Box<[(StakeAddress<MAINNET>, u64)]>,
    #[n(6)]
    pub update: Option<protocol::Update>,
    #[cbor(n(7), with = "minicbor::bytes")]
    pub data_hash: Option<Blake2b256Digest>,
    #[n(8)]
    pub validity_start: Option<u64>,
    #[n(9)]
    pub mint: Option<Asset<i64>>,
    #[cbor(n(11), with = "minicbor::bytes")]
    pub script_data_hash: Option<Blake2b256Digest>,
    #[cbor(n(13), with = "cbor_util::boxed_slice", has_nil)]
    pub collateral: Box<[Input]>,
    #[cbor(n(14), with = "cbor_util::boxed_slice::bytes", has_nil)]
    pub required_signers: Box<[Blake2b224Digest]>,
    #[n(16)]
    pub collateral_return: Option<Output<MAINNET>>,
    #[n(17)]
    pub total_collateral: Option<u64>,
    #[cbor(n(18), with = "cbor_util::boxed_slice", has_nil)]
    pub reference_inputs: Box<[Input]>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct Input {
    #[cbor(n(0), with = "minicbor::bytes")]
    pub id: Blake2b256Digest,
    #[n(1)]
    pub index: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode)]
#[cbor(map)]
pub struct Output<const MAINNET: bool> {
    #[n(0)]
    pub address: Address<MAINNET>,
    #[n(1)]
    pub amount: u64,
    #[n(2)]
    pub datum: Option<Datum>,
    #[cbor(n(3), with = "cbor_util::cbor_encoded", has_nil)]
    pub script_ref: Option<Script>,
}

impl<const M: bool, C> Decode<'_, C> for Output<M> {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        use minicbor::data::Type;
        match d.datatype()? {
            Type::Array | Type::ArrayIndef => {
                let len = d.array()?;
                match len {
                    Some(2 | 3) | None => {
                        let ouput = Output {
                            address: d.decode()?,
                            amount: d.decode()?,
                            datum: if len == Some(3)
                                || (len.is_none() && d.datatype()? != Type::Break)
                            {
                                Some(d.decode()?)
                            } else {
                                None
                            },
                            script_ref: None,
                        };
                        if len.is_none() {
                            if d.datatype()? != Type::Break {
                                return Err(minicbor::decode::Error::message(
                                    "Invalid array length",
                                ));
                            };
                            d.skip()?;
                        }
                        Ok(ouput)
                    }
                    _ => Err(minicbor::decode::Error::message("Invalid array length")),
                }
            }
            Type::Map | Type::MapIndef => {
                #[derive(Decode)]
                #[cbor(map)]
                struct BabbageOutput<const MAINNET: bool> {
                    #[n(0)]
                    pub address: Address<MAINNET>,
                    #[n(1)]
                    pub amount: u64,
                    #[n(2)]
                    pub datum: Option<Datum>,
                    #[cbor(n(3), with = "cbor_util::cbor_encoded", has_nil)]
                    pub script_ref: Option<Script>,
                }
                let BabbageOutput {
                    address,
                    amount,
                    datum,
                    script_ref,
                } = d.decode()?;
                Ok(Output {
                    address,
                    amount,
                    datum,
                    script_ref,
                })
            }
            t => Err(minicbor::decode::Error::type_mismatch(t)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[cbor(flat)]
pub enum Datum {
    #[n(0)]
    Hash(#[cbor(n(0), with = "minicbor::bytes")] Blake2b256Digest),
    #[n(1)]
    Data(#[n(0)] plutus::Data),
}
