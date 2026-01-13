use bip32::ExtendedVerifyingKey;
use minicbor::{CborLen, Decode, Encode};

use crate::{
    crypto::Signature,
    protocol,
    script::{native, plutus},
};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Set {
    #[n(0)]
    #[cbor(with = "cbor_util::set", has_nil)]
    pub verifying_keys: Box<[VerifyingKey]>,
    #[n(1)]
    #[cbor(with = "cbor_util::set", has_nil)]
    pub native_scripts: Box<[native::Script]>,
    #[n(2)]
    #[cbor(with = "cbor_util::set", has_nil)]
    pub bootstraps: Box<[Bootstrap]>,
    #[n(3)]
    #[cbor(with = "cbor_util::set", has_nil)]
    pub plutus_v1: Box<[plutus::Script]>,
    #[n(4)]
    #[cbor(with = "cbor_util::set", has_nil)]
    pub plutus_data: Box<[plutus::Data]>,
    #[n(5)]
    pub redeemers: Redeemers,
    #[n(6)]
    #[cbor(with = "cbor_util::set", has_nil)]
    pub plutus_v2: Box<[plutus::Script]>,
    #[n(7)]
    #[cbor(with = "cbor_util::set", has_nil)]
    pub plutus_v3: Box<[plutus::Script]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct VerifyingKey {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    pub vkey: crate::crypto::VerifyingKey,
    #[n(1)]
    #[cbor(with = "cbor_util::signature")]
    pub signature: Signature,
}

impl std::hash::Hash for VerifyingKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vkey.hash(state);
        state.write(self.signature.r_bytes());
        state.write(self.signature.s_bytes());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bootstrap {
    pub key: ExtendedVerifyingKey,
    pub signature: Signature,
    pub attributes: Box<[u8]>,
}

impl std::hash::Hash for Bootstrap {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        state.write(self.signature.r_bytes());
        state.write(self.signature.s_bytes());
        self.attributes.hash(state);
    }
}

impl<C> Encode<C> for Bootstrap {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(4)?;
        minicbor::bytes::encode(self.key.key_bytes(), e, ctx)?;
        cbor_util::signature::encode(&self.signature, e, &mut ())?;

        e.bytes(&self.key.chain_code)?.bytes(&self.attributes)?.ok()
    }
}

impl<C> Decode<'_, C> for Bootstrap {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        cbor_util::array_decode(
            4,
            |d| {
                let verifying_key: [u8; 32] = minicbor::bytes::decode(d, &mut ())?;
                let signature = cbor_util::signature::decode(d, &mut ())?;
                let chain_code: [u8; 32] = minicbor::bytes::decode(d, &mut ())?;
                let attributes: Vec<u8> = minicbor::bytes::decode(d, &mut ())?;

                let key = ExtendedVerifyingKey::new(verifying_key, chain_code).ok_or(
                    minicbor::decode::Error::message("Invalid verifying key curve point"),
                )?;

                Ok(Bootstrap {
                    key,
                    signature,
                    attributes: attributes.into_boxed_slice(),
                })
            },
            d,
        )
    }
}

impl<C> CborLen<C> for Bootstrap {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        let attributes: &minicbor::bytes::ByteSlice = (*self.attributes).into();
        let two_arrays_len = 2 * (32.cbor_len(ctx) + 32);

        4.cbor_len(ctx)
            + attributes.cbor_len(ctx)
            + cbor_util::signature::cbor_len(&self.signature, ctx)
            + two_arrays_len
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Redeemers(pub Box<[Redeemer]>);

impl<C> Encode<C> for Redeemers {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(self.0.len() as u64)?;
        self.0.iter().try_for_each(|r| {
            e.array(2)?.encode(&r.tag)?.encode(r.index)?;
            e.array(2)?.encode(&r.data)?.encode(&r.execution_units)?;
            Ok(())
        })?;
        Ok(())
    }

    fn is_nil(&self) -> bool {
        self.0.is_empty()
    }
}

impl<C> Decode<'_, C> for Redeemers {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::Array | minicbor::data::Type::ArrayIndef => {
                #[derive(Decode)]
                struct LegacyRedeemer {
                    #[n(0)]
                    pub tag: Tag,
                    #[n(1)]
                    pub index: u64,
                    #[n(2)]
                    pub data: plutus::Data,
                    #[n(3)]
                    pub execution_units: protocol::ExecutionUnits,
                }
                d.array_iter::<LegacyRedeemer>()?
                    .map(|r| {
                        r.map(
                            |LegacyRedeemer {
                                 tag,
                                 index,
                                 data,
                                 execution_units,
                             }| {
                                Redeemer {
                                    tag,
                                    index,
                                    data,
                                    execution_units,
                                }
                            },
                        )
                    })
                    .collect::<Result<Box<[_]>, _>>()
                    .map(Redeemers)
            }
            minicbor::data::Type::Map | minicbor::data::Type::MapIndef => {
                #[derive(Decode)]
                struct Key {
                    #[n(0)]
                    pub tag: Tag,
                    #[n(1)]
                    pub index: u64,
                }
                #[derive(Decode)]
                struct Value {
                    #[n(0)]
                    pub data: plutus::Data,
                    #[n(1)]
                    pub execution_units: protocol::ExecutionUnits,
                }
                d.map_iter::<Key, Value>()?
                    .map(|r| {
                        r.map(
                            |(
                                Key { tag, index },
                                Value {
                                    data,
                                    execution_units,
                                },
                            )| Redeemer {
                                tag,
                                index,
                                data,
                                execution_units,
                            },
                        )
                    })
                    .collect::<Result<Box<[_]>, _>>()
                    .map(Redeemers)
            }
            t => Err(minicbor::decode::Error::type_mismatch(t).at(d.position())),
        }
    }

    fn nil() -> Option<Self> {
        Some(Redeemers(Vec::new().into_boxed_slice()))
    }
}

impl<C> CborLen<C> for Redeemers {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        let len = self.0.len();
        len.cbor_len(ctx)
            + self
                .0
                .iter()
                .map(|r| {
                    2.cbor_len(ctx) * 2
                        + r.tag.cbor_len(ctx)
                        + r.index.cbor_len(ctx)
                        + r.data.cbor_len(ctx)
                        + r.execution_units.cbor_len(ctx)
                })
                .sum::<usize>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Redeemer {
    pub tag: Tag,
    pub index: u64,
    pub data: plutus::Data,
    pub execution_units: protocol::ExecutionUnits,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(index_only)]
pub enum Tag {
    #[n(0)]
    Spend,
    #[n(1)]
    Mint,
    #[n(2)]
    Certify,
    #[n(3)]
    Reward,
    #[n(4)]
    Vote,
    #[n(5)]
    Propose,
}
