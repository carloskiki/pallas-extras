use crate::{epoch, shelley::{UnitInterval, transaction::Coin}};

pub mod version;
use sparse_struct::SparseStruct;
pub use version::Version;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SparseStruct)]
#[struct_name = "Parameters"]
#[struct_derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Parameter {
    MinimumFeeA(Coin),
    MinimumFeeB(Coin),
    MaximumBlockBodySize(u32),
    MaximumTransactionSize(u32),
    MaximumBlockHeaderSize(u16),
    KeyDeposit(Coin),
    PoolDeposit(Coin),
    MaximumEpoch(epoch::Interval),
    StakePoolCountTarget(u16),
    PoolPledgeInfluence(UnitInterval),
    ExpansionRate(UnitInterval),
    TreasuryGrowthRate(UnitInterval),
    DecentralizationConstant(UnitInterval),
    ExtraEntropy(Option<[u8; 32]>),
    ProtocolVersion(Version),
    MinimumUtxoValue(Coin),
    MinimumPoolCost(Coin),
}


//     AdaPerUtxoByte(Coin),
//     ScriptCostModel(CostModels),
//     ExecutionCosts(ExecutionCosts),
//     MaxTxExecutionUnits(ExecutionUnits),
//     MaxBlockExecutionUnits(ExecutionUnits),
//     MaxValueSize(u32),
//     CollateralPercentage(u16),
//     MaxCollateralInputs(u16),
//     PoolVotingThresholds(pool::VotingThresholds),
//     DrepVotingThresholds(delegate_representative::VotingThresholds),
//     MinCommitteeSize(u16),
//     CommitteeTermLimit(epoch::Interval),
//     GovernanceActionValidityPeriod(epoch::Interval),
//     GovernanceActionDeposit(Coin),
//     DrepDeposit(Coin),
//     DrepInactivityPeriod(epoch::Interval),
//     /// Reference script cost per byte
//     ScriptReferenceCost(RealNumber),
// }
// 
// impl Parameter {
//     fn tag(&self) -> u8 {
//         match self {
//             Parameter::MinfeeA(_) => 0,
//             Parameter::MinfeeB(_) => 1,
//             Parameter::MaxBlockBodySize(_) => 2,
//             Parameter::MaxTransactionSize(_) => 3,
//             Parameter::MaxBlockHeaderSize(_) => 4,
//             Parameter::KeyDeposit(_) => 5,
//             Parameter::PoolDeposit(_) => 6,
//             Parameter::MaximumEpoch(_) => 7,
//             Parameter::StakePoolCountTarget(_) => 8,
//             Parameter::PoolPledgeInfluence(_) => 9,
//             Parameter::ExpansionRate(_) => 10,
//             Parameter::TreasuryGrowthRate(_) => 11,
//             Parameter::DecentralizationConstant(_) => 12,
//             Parameter::ExtraEntropy(_) => 13,
//             Parameter::ProtocolVersion(_) => 14,
//             Parameter::MinimumUtxoValue(_) => 15,
//             Parameter::MinimumPoolCost(_) => 16,
//             Parameter::AdaPerUtxoByte(_) => 17,
//             Parameter::ScriptCostModel(_) => 18,
//             Parameter::ExecutionCosts(_) => 19,
//             Parameter::MaxTxExecutionUnits(_) => 20,
//             Parameter::MaxBlockExecutionUnits(_) => 21,
//             Parameter::MaxValueSize(_) => 22,
//             Parameter::CollateralPercentage(_) => 23,
//             Parameter::MaxCollateralInputs(_) => 24,
//             Parameter::PoolVotingThresholds(_) => 25,
//             Parameter::DrepVotingThresholds(_) => 26,
//             Parameter::MinCommitteeSize(_) => 27,
//             Parameter::CommitteeTermLimit(_) => 28,
//             Parameter::GovernanceActionValidityPeriod(_) => 29,
//             Parameter::GovernanceActionDeposit(_) => 30,
//             Parameter::DrepDeposit(_) => 31,
//             Parameter::DrepInactivityPeriod(_) => 32,
//             Parameter::ScriptReferenceCost(_) => 33,
//         }
//     }
// }
// 
// impl<C> Encode<C> for Parameter {
//     fn encode<W: minicbor::encode::Write>(
//         &self,
//         e: &mut minicbor::Encoder<W>,
//         ctx: &mut C,
//     ) -> Result<(), minicbor::encode::Error<W::Error>> {
//         e.u8(self.tag())?;
//         match self {
//             Parameter::MinfeeA(v) => e.encode_with(v, ctx)?,
//             Parameter::MinfeeB(v) => e.encode_with(v, ctx)?,
//             Parameter::MaxBlockBodySize(v) => e.encode_with(v, ctx)?,
//             Parameter::MaxTransactionSize(v) => e.encode_with(v, ctx)?,
//             Parameter::MaxBlockHeaderSize(v) => e.encode_with(v, ctx)?,
//             Parameter::KeyDeposit(v) => e.encode_with(v, ctx)?,
//             Parameter::PoolDeposit(v) => e.encode_with(v, ctx)?,
//             Parameter::MaximumEpoch(v) => e.encode_with(v, ctx)?,
//             Parameter::StakePoolCountTarget(v) => e.encode_with(v, ctx)?,
//             Parameter::PoolPledgeInfluence(v) => e.encode_with(v, ctx)?,
//             Parameter::ExpansionRate(v) => e.encode_with(v, ctx)?,
//             Parameter::TreasuryGrowthRate(v) => e.encode_with(v, ctx)?,
//             Parameter::DecentralizationConstant(v) => e.encode_with(v, ctx)?,
//             Parameter::ExtraEntropy(v) => {
//                 nonce_encode(e, v)?;
//                 e
//             }
//             Parameter::ProtocolVersion(v) => e.encode_with(v, ctx)?,
//             Parameter::MinimumUtxoValue(v) => e.encode_with(v, ctx)?,
//             Parameter::MinimumPoolCost(v) => e.encode_with(v, ctx)?,
//             Parameter::AdaPerUtxoByte(v) => e.encode_with(v, ctx)?,
//             Parameter::ScriptCostModel(v) => e.encode_with(v, ctx)?,
//             Parameter::ExecutionCosts(v) => e.encode_with(v, ctx)?,
//             Parameter::MaxTxExecutionUnits(v) => e.encode_with(v, ctx)?,
//             Parameter::MaxBlockExecutionUnits(v) => e.encode_with(v, ctx)?,
//             Parameter::MaxValueSize(v) => e.encode_with(v, ctx)?,
//             Parameter::CollateralPercentage(v) => e.encode_with(v, ctx)?,
//             Parameter::MaxCollateralInputs(v) => e.encode_with(v, ctx)?,
//             Parameter::PoolVotingThresholds(v) => e.encode_with(v, ctx)?,
//             Parameter::DrepVotingThresholds(v) => e.encode_with(v, ctx)?,
//             Parameter::MinCommitteeSize(v) => e.encode_with(v, ctx)?,
//             Parameter::CommitteeTermLimit(v) => e.encode_with(v, ctx)?,
//             Parameter::GovernanceActionValidityPeriod(v) => e.encode_with(v, ctx)?,
//             Parameter::GovernanceActionDeposit(v) => e.encode_with(v, ctx)?,
//             Parameter::DrepDeposit(v) => e.encode_with(v, ctx)?,
//             Parameter::DrepInactivityPeriod(v) => e.encode_with(v, ctx)?,
//             Parameter::ScriptReferenceCost(v) => e.encode_with(v, ctx)?,
//         };
//         Ok(())
//     }
// }
// 
// impl<C> Decode<'_, C> for Parameter {
//     fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
//         let tag = d.u8()?;
//         match tag {
//             0 => Ok(Parameter::MinfeeA(d.decode()?)),
//             1 => Ok(Parameter::MinfeeB(d.decode()?)),
//             2 => Ok(Parameter::MaxBlockBodySize(d.decode()?)),
//             3 => Ok(Parameter::MaxTransactionSize(d.decode()?)),
//             4 => Ok(Parameter::MaxBlockHeaderSize(d.decode()?)),
//             5 => Ok(Parameter::KeyDeposit(d.decode()?)),
//             6 => Ok(Parameter::PoolDeposit(d.decode()?)),
//             7 => Ok(Parameter::MaximumEpoch(d.decode()?)),
//             8 => Ok(Parameter::StakePoolCountTarget(d.decode()?)),
//             9 => Ok(Parameter::PoolPledgeInfluence(d.decode()?)),
//             10 => Ok(Parameter::ExpansionRate(d.decode()?)),
//             11 => Ok(Parameter::TreasuryGrowthRate(d.decode()?)),
//             12 => Ok(Parameter::DecentralizationConstant(d.decode()?)),
//             13 => Ok(Parameter::ExtraEntropy(nonce_decode(d)?)),
//             14 => Ok(Parameter::ProtocolVersion(d.decode()?)),
//             15 => Ok(Parameter::MinimumUtxoValue(d.decode()?)),
//             16 => Ok(Parameter::MinimumPoolCost(d.decode()?)),
//             17 => Ok(Parameter::AdaPerUtxoByte(d.decode()?)),
//             18 => Ok(Parameter::ScriptCostModel(d.decode()?)),
//             19 => Ok(Parameter::ExecutionCosts(d.decode()?)),
//             20 => Ok(Parameter::MaxTxExecutionUnits(d.decode()?)),
//             21 => Ok(Parameter::MaxBlockExecutionUnits(d.decode()?)),
//             22 => Ok(Parameter::MaxValueSize(d.decode()?)),
//             23 => Ok(Parameter::CollateralPercentage(d.decode()?)),
//             24 => Ok(Parameter::MaxCollateralInputs(d.decode()?)),
//             25 => Ok(Parameter::PoolVotingThresholds(d.decode()?)),
//             26 => Ok(Parameter::DrepVotingThresholds(d.decode()?)),
//             27 => Ok(Parameter::MinCommitteeSize(d.decode()?)),
//             28 => Ok(Parameter::CommitteeTermLimit(d.decode()?)),
//             29 => Ok(Parameter::GovernanceActionValidityPeriod(d.decode()?)),
//             30 => Ok(Parameter::GovernanceActionDeposit(d.decode()?)),
//             31 => Ok(Parameter::DrepDeposit(d.decode()?)),
//             32 => Ok(Parameter::DrepInactivityPeriod(d.decode()?)),
//             33 => Ok(Parameter::ScriptReferenceCost(d.decode()?)),
//             _ => Err(minicbor::decode::Error::tag_mismatch(
//                 minicbor::data::Tag::new(tag as u64),
//             )),
//         }
//     }
// }
// 
// impl<C> CborLen<C> for Parameter {
//     fn cbor_len(&self, ctx: &mut C) -> usize {
//         self.tag().cbor_len(ctx)
//             + match self {
//                 Parameter::MinfeeA(v) => v.cbor_len(ctx),
//                 Parameter::MinfeeB(v) => v.cbor_len(ctx),
//                 Parameter::MaxBlockBodySize(v) => v.cbor_len(ctx),
//                 Parameter::MaxTransactionSize(v) => v.cbor_len(ctx),
//                 Parameter::MaxBlockHeaderSize(v) => v.cbor_len(ctx),
//                 Parameter::KeyDeposit(v) => v.cbor_len(ctx),
//                 Parameter::PoolDeposit(v) => v.cbor_len(ctx),
//                 Parameter::MaximumEpoch(v) => v.cbor_len(ctx),
//                 Parameter::StakePoolCountTarget(v) => v.cbor_len(ctx),
//                 Parameter::PoolPledgeInfluence(v) => v.cbor_len(ctx),
//                 Parameter::ExpansionRate(v) => v.cbor_len(ctx),
//                 Parameter::TreasuryGrowthRate(v) => v.cbor_len(ctx),
//                 Parameter::DecentralizationConstant(v) => v.cbor_len(ctx),
//                 Parameter::ExtraEntropy(v) => nonce_len(v),
//                 Parameter::ProtocolVersion(v) => v.cbor_len(ctx),
//                 Parameter::MinimumUtxoValue(v) => v.cbor_len(ctx),
//                 Parameter::MinimumPoolCost(v) => v.cbor_len(ctx),
//                 Parameter::AdaPerUtxoByte(v) => v.cbor_len(ctx),
//                 Parameter::ScriptCostModel(v) => v.cbor_len(ctx),
//                 Parameter::ExecutionCosts(v) => v.cbor_len(ctx),
//                 Parameter::MaxTxExecutionUnits(v) => v.cbor_len(ctx),
//                 Parameter::MaxBlockExecutionUnits(v) => v.cbor_len(ctx),
//                 Parameter::MaxValueSize(v) => v.cbor_len(ctx),
//                 Parameter::CollateralPercentage(v) => v.cbor_len(ctx),
//                 Parameter::MaxCollateralInputs(v) => v.cbor_len(ctx),
//                 Parameter::PoolVotingThresholds(v) => v.cbor_len(ctx),
//                 Parameter::DrepVotingThresholds(v) => v.cbor_len(ctx),
//                 Parameter::MinCommitteeSize(v) => v.cbor_len(ctx),
//                 Parameter::CommitteeTermLimit(v) => v.cbor_len(ctx),
//                 Parameter::GovernanceActionValidityPeriod(v) => v.cbor_len(ctx),
//                 Parameter::GovernanceActionDeposit(v) => v.cbor_len(ctx),
//                 Parameter::DrepDeposit(v) => v.cbor_len(ctx),
//                 Parameter::DrepInactivityPeriod(v) => v.cbor_len(ctx),
//                 Parameter::ScriptReferenceCost(v) => v.cbor_len(ctx),
//             }
//     }
// }
// 
// fn nonce_encode<W: minicbor::encode::Write>(
//     e: &mut minicbor::Encoder<W>,
//     n: &Option<[u8; 32]>,
// ) -> Result<(), minicbor::encode::Error<W::Error>> {
//     match n {
//         Some(bytes) => e.array(2)?.u8(1)?.bytes(bytes)?.ok(),
//         None => e.array(1)?.u8(0)?.ok(),
//     }
// }
// 
// fn nonce_decode(
//     d: &mut minicbor::Decoder<'_>,
// ) -> Result<Option<[u8; 32]>, minicbor::decode::Error> {
//     let array_len = d.array()?;
// 
//     let tag = d.u8()?;
//     let nonce = match tag {
//         0 if array_len.is_none_or(|l| l == 1) => None,
//         1 if array_len.is_none_or(|l| l == 2) => {
//             let bytes = d.bytes()?;
//             Some(bytes.try_into().map_err(minicbor::decode::Error::custom)?)
//         }
//         _ => {
//             return Err(minicbor::decode::Error::message(
//                 "array len incorrect or tag incorrect",
//             ));
//         }
//     };
// 
//     if array_len.is_none() && d.datatype()? != minicbor::data::Type::Break {
//         return Err(minicbor::decode::Error::message("invalid array len"));
//     }
// 
//     Ok(nonce)
// }
// 
// fn nonce_len(nonce: &Option<[u8; 32]>) -> usize {
//     match nonce {
//         Some(bytes) => 2.cbor_len(&mut ()) + 1.cbor_len(&mut ()) + bytes.cbor_len(&mut ()),
//         None => 1.cbor_len(&mut ()) + 0.cbor_len(&mut ()),
//     }
// }
// 
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, CborLen)]
// #[cbor(transparent)]
// pub struct ParameterUpdate(pub Box<[Parameter]>);
// 
// impl<C> Encode<C> for ParameterUpdate {
//     fn encode<W: minicbor::encode::Write>(
//         &self,
//         e: &mut minicbor::Encoder<W>,
//         _: &mut C,
//     ) -> Result<(), minicbor::encode::Error<W::Error>> {
//         e.map(self.0.len() as u64)?;
//         for param in &self.0 {
//             e.encode(param)?;
//         }
//         Ok(())
//     }
// }
// 
// impl<C> Decode<'_, C> for ParameterUpdate {
//     fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
//         let map_len = d.map()?;
//         let mut vec: Vec<Parameter>;
// 
//         if let Some(map_len) = map_len {
//             vec = Vec::with_capacity(map_len as usize);
//             for _ in 0..map_len {
//                 vec.push(d.decode()?);
//             }
//         } else {
//             vec = Vec::new();
//             while d.datatype()? != minicbor::data::Type::Break {
//                 vec.push(d.decode()?);
//             }
//         };
// 
//         Ok(ParameterUpdate(vec.into()))
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
// pub struct ExecutionUnits {
//     #[n(0)]
//     pub memory: u64,
//     #[n(1)]
//     pub step: u64,
// }
// 
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
// pub struct ExecutionCosts {
//     #[n(0)]
//     pub memory: RealNumber,
//     #[n(1)]
//     pub step: RealNumber,
// }

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, CborLen)]
// #[cbor(transparent)]
// pub struct CostModels(pub Box<[CostModel]>);

// impl<C> Encode<C> for CostModels {
//     fn encode<W: minicbor::encode::Write>(
//         &self,
//         e: &mut minicbor::Encoder<W>,
//         ctx: &mut C,
//     ) -> Result<(), minicbor::encode::Error<W::Error>> {
//         e.map(self.0.len() as u64)?;
//         for model in &self.0 {
//             match model {
//                 CostModel::PlutusV1(v) => e.u8(0)?.encode_with(v, ctx)?,
//                 CostModel::PlutusV2(v) => e.u8(1)?.encode_with(v, ctx)?,
//                 CostModel::PlutusV3(v) => e.u8(2)?.encode_with(v, ctx)?,
//             };
//         }
//         Ok(())
//     }
// }
// 
// impl<C> Decode<'_, C> for CostModels {
//     fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
//         d.map_iter::<u8, Vec<i64>>()?
//             .map(|v| {
//                 let (tag, ints) = v?;
//                 let model = match tag {
//                     0 => CostModel::PlutusV1(Box::new(
//                         <[i64; 166]>::try_from(
//                             ints.get(..166)
//                                 .ok_or(minicbor::decode::Error::message("Invalid array length"))?,
//                         )
//                         .unwrap(),
//                     )),
//                     1 => CostModel::PlutusV2(Box::new(
//                         <[i64; 175]>::try_from(
//                             ints.get(..175)
//                                 .ok_or(minicbor::decode::Error::message("Invalid array length"))?,
//                         )
//                         .unwrap(),
//                     )),
//                     2 => CostModel::PlutusV3(Box::new(
//                         <[i64; 233]>::try_from(
//                             ints.get(..233)
//                                 .ok_or(minicbor::decode::Error::message("Invalid array length"))?,
//                         )
//                         .unwrap(),
//                     )),
// 
//                     t => {
//                         return Err(minicbor::decode::Error::tag_mismatch(
//                             minicbor::data::Tag::new(t as u64),
//                         ));
//                     }
//                 };
//                 Ok(model)
//             })
//             .collect::<Result<_, _>>()
//             .map(CostModels)
//     }
// }
// 
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub enum CostModel {
//     PlutusV1(Box<[i64; 166]>),
//     PlutusV2(Box<[i64; 175]>),
//     PlutusV3(Box<[i64; 233]>),
// }
// 
// impl CostModel {
//     fn tag(&self) -> u8 {
//         match self {
//             CostModel::PlutusV1(_) => 0,
//             CostModel::PlutusV2(_) => 1,
//             CostModel::PlutusV3(_) => 2,
//         }
//     }
// }
// 
// impl<C> Encode<C> for CostModel {
//     fn encode<W: minicbor::encode::Write>(
//         &self,
//         e: &mut minicbor::Encoder<W>,
//         ctx: &mut C,
//     ) -> Result<(), minicbor::encode::Error<W::Error>> {
//         e.u8(self.tag())?;
//         match self {
//             CostModel::PlutusV1(v) => e.encode_with(v, ctx)?.ok(),
//             CostModel::PlutusV2(v) => e.encode_with(v, ctx)?.ok(),
//             CostModel::PlutusV3(v) => e.encode_with(v, ctx)?.ok(),
//         }
//     }
// }
// 
// impl<C> CborLen<C> for CostModel {
//     fn cbor_len(&self, ctx: &mut C) -> usize {
//         self.tag().cbor_len(ctx)
//             + match self {
//                 CostModel::PlutusV1(v) => v.cbor_len(ctx),
//                 CostModel::PlutusV2(v) => v.cbor_len(ctx),
//                 CostModel::PlutusV3(v) => v.cbor_len(ctx),
//             }
//     }
// }
// 
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
// pub struct Update {
//     #[cbor(n(0), with = "cbor_util::list_as_map::key_bytes")]
//     pub proposed: Box<[(Blake2b224Digest, ParameterUpdate)]>,
//     #[n(1)]
//     pub epoch: u64,
// }
// 
// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
// #[cbor(tag(30))] // TODO: This isn't a real number, handle all its variants properly.
// pub struct RealNumber {
//     #[n(0)]
//     pub numerator: u64,
//     #[n(1)]
//     pub denominator: u64,
// }
