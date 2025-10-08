use std::num::NonZeroU64;

use cbor_util::{bytes_iter_collect, str_iter_collect};
use minicbor::{CborLen, Decode, Encode};

use super::{
    address::{Address, shelley::StakeAddress},
    certificate, protocol, witness,
};
use crate::{
    asset::Asset,
    crypto::{Blake2b224Digest, Blake2b256Digest},
    governance,
    script::{Script, native, plutus},
    slot,
};

pub type Id = Blake2b256Digest;
pub type Coin = u64;

#[derive(Debug, Clone, PartialEq, Eq, Encode)]
pub struct Transaction {
    #[n(0)]
    pub body: Body,
    #[n(1)]
    pub witness_set: witness::Set,
    #[n(2)]
    pub valid_scripts: bool,
    #[n(3)]
    pub data: Option<Data>,
}

impl<C> Decode<'_, C> for Transaction {
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

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, CborLen)]
#[cbor(map, tag(259))]
pub struct Data {
    #[cbor(n(0), with = "cbor_util::list_as_map", has_nil)]
    pub metadata: Box<[(u64, Metadatum)]>,
    #[cbor(n(1), with = "cbor_util::boxed_slice", has_nil)]
    pub auxiliary_scripts: Box<[native::Script]>,
    #[cbor(n(2), with = "cbor_util::boxed_slice", has_nil)]
    pub plutus_v1: Box<[plutus::Script]>,
    #[cbor(n(3), with = "cbor_util::boxed_slice", has_nil)]
    pub plutus_v2: Box<[plutus::Script]>,
    #[cbor(n(4), with = "cbor_util::boxed_slice", has_nil)]
    pub plutus_v3: Box<[plutus::Script]>,
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
                    #[cbor(n(4), with = "cbor_util::boxed_slice", has_nil)]
                    plutus_v3: Box<[plutus::Script]>,
                }

                let DataAlonzo {
                    metadata,
                    auxiliary_scripts,
                    plutus_v1,
                    plutus_v2,
                    plutus_v3,
                } = d.decode()?;

                Ok(Data {
                    metadata,
                    auxiliary_scripts,
                    plutus_v1,
                    plutus_v2,
                    plutus_v3,
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
            t => Err(minicbor::decode::Error::type_mismatch(t).at(d.position())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Metadatum {
    Integer(minicbor::data::Int),
    Bytes(Box<[u8]>), // TODO: max len = 64
    Text(Box<str>),   // TODO: max len = 64
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
            Metadatum::Integer(i) => e.int(*i)?.ok(),
            Metadatum::Bytes(b) => e.bytes(b)?.ok(),
            Metadatum::Text(s) => e.str(s)?.ok(),
            Metadatum::Array(a) => e.encode(a)?.ok(),
            Metadatum::Map(m) => cbor_util::list_as_map::encode(m, e, ctx),
        }
    }
}

impl<C> Decode<'_, C> for Metadatum {
    fn decode(d: &mut minicbor::Decoder<'_>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        use minicbor::data::Type;
        match d.datatype()? {
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::Int => Ok(Metadatum::Integer(d.int()?)),
            Type::Bytes | Type::BytesIndef => {
                Ok(Metadatum::Bytes(bytes_iter_collect(d.bytes_iter()?)?))
            }
            Type::String | Type::StringIndef => {
                Ok(Metadatum::Text(str_iter_collect(d.str_iter()?)?))
            }
            Type::Array | Type::ArrayIndef => Ok(Metadatum::Array(
                d.array_iter()?.collect::<Result<Box<[_]>, _>>()?,
            )),
            Type::Map | Type::MapIndef => {
                Ok(Metadatum::Map(cbor_util::list_as_map::decode(d, ctx)?))
            }
            t => Err(minicbor::decode::Error::type_mismatch(t).at(d.position())),
        }
    }
}

impl<C> CborLen<C> for Metadatum {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        match self {
            Metadatum::Integer(i) => i.cbor_len(ctx),
            Metadatum::Bytes(bytes) => minicbor::bytes::cbor_len(bytes, ctx),
            Metadatum::Text(string) => string.cbor_len(ctx),
            Metadatum::Array(metadatums) => metadatums.cbor_len(ctx),
            Metadatum::Map(items) => cbor_util::list_as_map::cbor_len(items, ctx),
        }
    }
}

// TODO: All optional fields should be in a Set instead of in the struct, as with Protocol Update
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Body {
    #[cbor(n(0), with = "cbor_util::set")]
    pub inputs: Box<[Input]>,
    #[cbor(n(1), with = "cbor_util::boxed_slice")]
    pub outputs: Box<[Output]>,
    #[n(2)]
    pub fee: Coin,
    #[n(3)]
    pub ttl: Option<slot::Number>,
    #[cbor(n(4), with = "cbor_util::set", has_nil)]
    pub certificates: Box<[certificate::Certificate]>,
    #[cbor(n(5), with = "cbor_util::list_as_map", has_nil)]
    pub withdrawals: Box<[(StakeAddress, Coin)]>,
    #[n(6)]
    pub update: Option<protocol::Update>, // TODO: No longer present in conway
    #[cbor(n(7), with = "minicbor::bytes")]
    pub data_hash: Option<Blake2b256Digest>,
    #[n(8)]
    pub validity_start: Option<slot::Number>,
    #[n(9)] // TODO: should be NonZeroI64
    pub mint: Option<Asset<i64>>,
    #[cbor(n(11), with = "minicbor::bytes")]
    pub script_data_hash: Option<Blake2b256Digest>,
    #[cbor(n(13), with = "cbor_util::set", has_nil)]
    pub collateral: Box<[Input]>,
    #[cbor(n(14), with = "cbor_util::set::bytes", has_nil)]
    pub required_signers: Box<[Blake2b224Digest]>,
    #[cbor(n(15), with = "network_id", has_nil)]
    pub testnet: Option<bool>,
    #[n(16)]
    pub collateral_return: Option<Output>,
    #[n(17)]
    pub total_collateral: Option<Coin>,
    #[cbor(n(18), with = "cbor_util::set", has_nil)]
    pub reference_inputs: Box<[Input]>,

    #[cbor(n(19), with = "cbor_util::list_as_map", has_nil)]
    pub voting_procedures: Box<[(governance::voting::Voter, governance::voting::Set)]>,
    #[cbor(n(20), with = "cbor_util::set", has_nil)]
    pub proposal_procedures: Box<[governance::Proposal]>,
    #[n(21)]
    pub treasury_donation: Option<Coin>, // NOTE: We may have swapped 21 and 22, specification is
    // unclear on which is which
    #[n(22)]
    pub current_treasury: Option<NonZeroU64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Input {
    #[cbor(n(0), with = "minicbor::bytes")]
    pub id: Id,
    #[n(1)]
    pub index: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, CborLen)]
#[cbor(map)]
pub struct Output {
    #[n(0)]
    pub address: Address,
    #[n(1)]
    pub amount: Value,
    #[n(2)]
    pub datum: Option<Datum>,
    #[cbor(n(3), with = "cbor_util::cbor_encoded", has_nil)]
    pub script_ref: Option<Script>,
}

impl<C> Decode<'_, C> for Output {
    fn decode(d: &mut minicbor::Decoder<'_>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        use minicbor::data::Type;
        match d.datatype()? {
            Type::Array | Type::ArrayIndef => {
                let len = d.array()?;
                match len {
                    Some(2 | 3) | None => {
                        let output = Output {
                            address: d.decode()?,
                            amount: d.decode()?,
                            datum: if len == Some(3)
                                || (len.is_none() && d.datatype()? != Type::Break)
                            {
                                Some(Datum::Hash(minicbor::bytes::decode(d, ctx)?))
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
                        Ok(output)
                    }
                    _ => Err(minicbor::decode::Error::message("Invalid array length")),
                }
            }
            Type::Map | Type::MapIndef => {
                #[derive(Decode)]
                #[cbor(map)]
                struct BabbageOutput {
                    #[n(0)]
                    pub address: Address,
                    #[n(1)]
                    pub amount: Value,
                    #[n(2)]
                    pub datum: Option<Datum>,
                    #[cbor(n(3), with = "cbor_util::cbor_encoded")]
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
            t => Err(minicbor::decode::Error::type_mismatch(t)
                .at(d.position())
                .with_message(std::panic::Location::caller())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Ada(Coin),
    Other {
        ada: Coin,
        assets: Asset<NonZeroU64>,
    },
}

impl<C> Encode<C> for Value {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Value::Ada(ada) => e.encode(ada),
            Value::Other { ada, assets } => e.array(2)?.encode(ada)?.encode(assets),
        }?
        .ok()
    }
}

impl<C> Decode<'_, C> for Value {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::U8
            | minicbor::data::Type::U16
            | minicbor::data::Type::U32
            | minicbor::data::Type::U64 => Ok(Value::Ada(d.u64()?)),
            minicbor::data::Type::Array | minicbor::data::Type::ArrayIndef => {
                cbor_util::array_decode(
                    2,
                    |d| {
                        Ok(Value::Other {
                            ada: d.u64()?,
                            assets: {
                                // NOTE: We accept assets with count 0, but we filter them out.
                                // In Conway and beyond, assets with value 0 are forbidden.
                                let with_zeros: Asset<u64> = d.decode()?;
                                Asset(
                                    with_zeros
                                        .0
                                        .into_iter()
                                        .filter_map(|(hash, bundle)| {
                                            let bundle = bundle
                                                .0
                                                .into_iter()
                                                .filter_map(|(name, value)| {
                                                    NonZeroU64::new(value).map(|v| (name, v))
                                                })
                                                .collect::<Box<[_]>>();
                                            (!bundle.is_empty())
                                                .then_some((hash, crate::asset::Bundle(bundle)))
                                        })
                                        .collect::<Box<[_]>>(),
                                )
                            },
                        })
                    },
                    d,
                )
            }
            t => Err(minicbor::decode::Error::type_mismatch(t).at(d.position())),
        }
    }
}

impl<C> CborLen<C> for Value {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        match self {
            Value::Ada(ada) => ada.cbor_len(ctx),
            Value::Other { ada, assets } => {
                2.cbor_len(ctx) + ada.cbor_len(ctx) + assets.cbor_len(ctx)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(flat)]
pub enum Datum {
    #[n(0)]
    Hash(#[cbor(n(0), with = "minicbor::bytes")] Blake2b256Digest),
    #[n(1)]
    Data(#[cbor(n(0), with = "cbor_util::cbor_encoded")] plutus::data::Data),
}

mod network_id {
    use minicbor::{CborLen, Decoder, Encoder, decode as de, encode as en};

    pub fn encode<C, W: en::Write>(
        value: &Option<bool>,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        if let Some(value) = value {
            e.u8(*value as u8)?.ok()
        } else {
            e.null()?.ok()
        }
    }

    pub fn decode<Ctx>(d: &mut Decoder<'_>, _: &mut Ctx) -> Result<Option<bool>, de::Error> {
        match d.u8() {
            Ok(v) => Ok(Some(v != 0)),
            Err(e) if e.is_type_mismatch() => {
                d.null()?;
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    pub fn cbor_len<C>(value: &Option<bool>, ctx: &mut C) -> usize {
        value.map(|v| (v as u8).cbor_len(ctx)).unwrap_or(1)
    }

    pub fn nil() -> Option<Option<bool>> {
        Some(None)
    }

    pub fn is_nil(v: &Option<bool>) -> bool {
        v.is_none()
    }
}
