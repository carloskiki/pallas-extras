use std::fmt::Debug;

use minicbor::{Decode, Encode};

use crate::crypto::Blake2b224Digest;

use super::RealNumber;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct Version {
    #[n(0)]
    pub major: MajorVersion,
    #[n(1)]
    pub minor: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[cbor(index_only)]
pub enum MajorVersion {
    // The Byron Era
    #[n(1)]
    Byron,
    // The Shelley Era
    #[n(2)]
    Shelley,
    #[n(3)]
    Allegra,
    #[n(4)]
    Mary,
    #[n(5)]
    Alonzo,
    /// Part of the Alonzo Era
    #[n(6)]
    Lobster,
    /// Part of the Babbage Era
    #[n(7)]
    Vasil,
    /// Part of the Babbage Era
    #[n(8)]
    Valentine,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Parameter {
    MinfeeA(u64),
    MinfeeB(u64),
    MaxBlockBodySize(u32),
    MaxTransactionSize(u32),
    MaxBlockHeaderSize(u16),
    KeyDeposit(u64),
    PoolDeposit(u64),
    MaximumEpoch(u64),
    StakePoolCountTarget(u16),
    PoolPledgeInfluence(RealNumber),
    ExpansionRate(RealNumber),
    TreasuryGrowthRate(RealNumber),
    DecentralizationConstant(RealNumber),
    ExtraEntropy(Option<[u8; 32]>),
    ProtocolVersion(Version),
    MinimumUtxoValue(u64),
    MinimumPoolCost(u64),
    AdaPerUtxoByte(u64),
    ScriptCostModel(CostModels),
    ExecutionCosts(ExecutionCosts),
    MaxTxExecutionUnits(ExecutionUnits),
    MaxBlockExecutionUnits(ExecutionUnits),
    MaxValueSize(u64),
    CollateralPercentage(u64),
    MaxCollateralInputs(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParameterUpdate(pub Box<[Parameter]>);

impl<C> Encode<C> for ParameterUpdate {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(self.0.len() as u64)?;
        for param in &self.0 {
            match param {
                Parameter::MinfeeA(v) => e.u8(0)?.encode_with(v, ctx)?,
                Parameter::MinfeeB(v) => e.u8(1)?.encode_with(v, ctx)?,
                Parameter::MaxBlockBodySize(v) => e.u8(2)?.encode_with(v, ctx)?,
                Parameter::MaxTransactionSize(v) => e.u8(3)?.encode_with(v, ctx)?,
                Parameter::MaxBlockHeaderSize(v) => e.u8(4)?.encode_with(v, ctx)?,
                Parameter::KeyDeposit(v) => e.u8(5)?.encode_with(v, ctx)?,
                Parameter::PoolDeposit(v) => e.u8(6)?.encode_with(v, ctx)?,
                Parameter::MaximumEpoch(v) => e.u8(7)?.encode_with(v, ctx)?,
                Parameter::StakePoolCountTarget(v) => e.u8(8)?.encode_with(v, ctx)?,
                Parameter::PoolPledgeInfluence(v) => e.u8(9)?.encode_with(v, ctx)?,
                Parameter::ExpansionRate(v) => e.u8(10)?.encode_with(v, ctx)?,
                Parameter::TreasuryGrowthRate(v) => e.u8(11)?.encode_with(v, ctx)?,
                Parameter::DecentralizationConstant(v) => e.u8(12)?.encode_with(v, ctx)?,
                Parameter::ExtraEntropy(v) => e.u8(13)?.encode_with(v, ctx)?,
                Parameter::ProtocolVersion(v) => e.u8(14)?.encode_with(v, ctx)?,
                Parameter::MinimumUtxoValue(v) => e.u8(15)?.encode_with(v, ctx)?,
                Parameter::MinimumPoolCost(v) => e.u8(16)?.encode_with(v, ctx)?,
                Parameter::AdaPerUtxoByte(v) => e.u8(17)?.encode_with(v, ctx)?,
                Parameter::ScriptCostModel(v) => e.u8(18)?.encode_with(v, ctx)?,
                Parameter::ExecutionCosts(v) => e.u8(19)?.encode_with(v, ctx)?,
                Parameter::MaxTxExecutionUnits(v) => e.u8(20)?.encode_with(v, ctx)?,
                Parameter::MaxBlockExecutionUnits(v) => e.u8(21)?.encode_with(v, ctx)?,
                Parameter::MaxValueSize(v) => e.u8(22)?.encode_with(v, ctx)?,
                Parameter::CollateralPercentage(v) => e.u8(23)?.encode_with(v, ctx)?,
                Parameter::MaxCollateralInputs(v) => e.u8(24)?.encode_with(v, ctx)?,
            };
        }
        Ok(())
    }
}

impl<C> Decode<'_, C> for ParameterUpdate {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        fn decode_update(
            d: &mut minicbor::Decoder<'_>,
        ) -> Result<Parameter, minicbor::decode::Error> {
            let tag = d.u8()?;
            match tag {
                0 => Ok(Parameter::MinfeeA(d.decode()?)),
                1 => Ok(Parameter::MinfeeB(d.decode()?)),
                2 => Ok(Parameter::MaxBlockBodySize(d.decode()?)),
                3 => Ok(Parameter::MaxTransactionSize(d.decode()?)),
                4 => Ok(Parameter::MaxBlockHeaderSize(d.decode()?)),
                5 => Ok(Parameter::KeyDeposit(d.decode()?)),
                6 => Ok(Parameter::PoolDeposit(d.decode()?)),
                7 => Ok(Parameter::MaximumEpoch(d.decode()?)),
                8 => Ok(Parameter::StakePoolCountTarget(d.decode()?)),
                9 => Ok(Parameter::PoolPledgeInfluence(d.decode()?)),
                10 => Ok(Parameter::ExpansionRate(d.decode()?)),
                11 => Ok(Parameter::TreasuryGrowthRate(d.decode()?)),
                12 => Ok(Parameter::DecentralizationConstant(d.decode()?)),
                13 => Ok(Parameter::ExtraEntropy(d.decode()?)),
                14 => Ok(Parameter::ProtocolVersion(d.decode()?)),
                15 => Ok(Parameter::MinimumUtxoValue(d.decode()?)),
                16 => Ok(Parameter::MinimumPoolCost(d.decode()?)),
                17 => Ok(Parameter::AdaPerUtxoByte(d.decode()?)),
                18 => Ok(Parameter::ScriptCostModel(d.decode()?)),
                19 => Ok(Parameter::ExecutionCosts(d.decode()?)),
                20 => Ok(Parameter::MaxTxExecutionUnits(d.decode()?)),
                21 => Ok(Parameter::MaxBlockExecutionUnits(d.decode()?)),
                22 => Ok(Parameter::MaxValueSize(d.decode()?)),
                23 => Ok(Parameter::CollateralPercentage(d.decode()?)),
                24 => Ok(Parameter::MaxCollateralInputs(d.decode()?)),
                _ => Err(minicbor::decode::Error::tag_mismatch(minicbor::data::Tag::new(tag as u64))),
            }
        }
        let map_len = d.map()?;
        let mut vec;
        
        if let Some(map_len) = map_len {
            vec = Vec::with_capacity(map_len as usize);
            for _ in 0..map_len {
                vec.push(decode_update(d)?);
            }
        } else {
            vec = Vec::new();
            while d.datatype()? != minicbor::data::Type::Break {
                vec.push(decode_update(d)?);
            }
        };
        
        Ok(ParameterUpdate(vec.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ExecutionUnits {
    #[n(0)]
    pub memory: u64,
    #[n(1)]
    pub step: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ExecutionCosts {
    #[n(0)]
    pub memory: RealNumber,
    #[n(1)]
    pub step: RealNumber,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CostModels(pub Box<[CostModel]>);

impl<C> Encode<C> for CostModels {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(self.0.len() as u64)?;
        for model in &self.0 {
            match model {
                CostModel::PlutusV1(v) => e.u8(0)?.encode_with(v, ctx)?,
                CostModel::PlutusV2(v) => e.u8(1)?.encode_with(v, ctx)?,
            };
        }
        Ok(())
    }
}

impl<C> Decode<'_, C> for CostModels {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        d.map_iter::<u8, Vec<i64>>()?
            .map(|v| {
                let (tag, ints) = v?;
                let model = match tag {
                    0 => CostModel::PlutusV1(ints.try_into().unwrap()),
                    1 => CostModel::PlutusV2(ints.try_into().unwrap()),
                    t => {
                        return Err(minicbor::decode::Error::tag_mismatch(
                            minicbor::data::Tag::new(t as u64),
                        ));
                    }
                };
                Ok(model)
            })
            .collect::<Result<_, _>>()
            .map(CostModels)
    }
}

pub enum Language {
    PlutusV1,
    PlutusV2,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CostModel {
    PlutusV1([i64; 166]),
    PlutusV2([i64; 175]),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct Update {
    #[n(0)]
    #[cbor(with = "crate::cbor::list_as_map")]
    pub proposed: Box<[(Blake2b224Digest, ParameterUpdate)]>,
    #[n(1)]
    pub epoch: u64,
}
