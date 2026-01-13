use crate::{
    epoch,
    conway::{UnitInterval, transaction::Coin},
};
use sparse_struct::SparseStruct;
use tinycbor::{
    CborLen, Decode, Encode, Encoder, Write,
    collections::{self, fixed, map},
    primitive,
};
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod version;
pub use version::Version;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, SparseStruct, Encode, Decode, CborLen,
)]
#[struct_name = "Parameters"]
#[struct_derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cbor(naked)]
pub enum Parameter {
    #[n(0)]
    MinimumFeeA(Coin),
    #[n(1)]
    MinimumFeeB(Coin),
    #[n(2)]
    MaximumBlockBodySize(u32),
    #[n(3)]
    MaximumTransactionSize(u32),
    #[n(4)]
    MaximumBlockHeaderSize(u16),
    #[n(5)]
    KeyDeposit(Coin),
    #[n(6)]
    PoolDeposit(Coin),
    #[n(7)]
    MaximumEpoch(epoch::Interval),
    #[n(8)]
    StakePoolCountTarget(u16),
    #[n(9)]
    PoolPledgeInfluence(UnitInterval),
    #[n(10)]
    ExpansionRate(UnitInterval),
    #[n(11)]
    TreasuryGrowthRate(UnitInterval),
    #[n(12)]
    DecentralizationConstant(UnitInterval),
    #[n(13)]
    ExtraEntropy(#[cbor(with = "cbor_util::option::Array<[u8; 32], true>")] Option<[u8; 32]>),
    #[n(14)]
    ProtocolVersion(Version),
    #[n(15)]
    MinimumUtxoValue(Coin),
    #[n(16)]
    MinimumPoolCost(Coin),
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
}

impl CborLen for Parameters {
    fn cbor_len(&self) -> usize {
        let params = self.as_ref();
        params.len().cbor_len() + params.iter().map(|param| param.cbor_len()).sum::<usize>()
    }
}

impl Encode for Parameters {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        let params = self.as_ref();
        e.map(params.len())?;
        params.iter().try_for_each(|param| param.encode(e))
    }
}

impl Decode<'_> for Parameters {
    type Error = collections::Error<fixed::Error<map::Error<primitive::Error, Error>>>;

    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        let decode_param = |d: &mut tinycbor::Decoder<'_>| {
            Parameter::decode(d).map_err(|e| {
                collections::Error::Element(match e {
                    tinycbor::tag::Error::Malformed(error) => {
                        fixed::Error::Inner(map::Error::Key(error))
                    }
                    tinycbor::tag::Error::InvalidTag => fixed::Error::Surplus,
                    tinycbor::tag::Error::Inner(inner) => {
                        fixed::Error::Inner(map::Error::Value(inner))
                    }
                })
            })
        };

        if let Some(len) = d.map_visitor()?.remaining() {
            for _ in 0..len {
                let param = decode_param(d)?;
                if !params.insert(param) {
                    return Err(collections::Error::Element(fixed::Error::Surplus));
                }
            }
        } else {
            while d.datatype()? != tinycbor::Type::Break {
                let param = decode_param(d)?;
                if !params.insert(param) {
                    return Err(collections::Error::Element(fixed::Error::Surplus));
                }
            }
            d.next().expect("found break").expect("valid break");
        };
        Ok(params)
    }
}

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
