use crate::{
    epoch,
    shelley::{UnitInterval, transaction::Coin},
};
use sparse_struct::SparseStruct;
use tinycbor::{
    CborLen, Decode, Encode, Encoder, Write,
    container::{self, bounded, map},
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
    type Error = container::Error<bounded::Error<map::Error<primitive::Error, Error>>>;

    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        let decode_param = |d: &mut tinycbor::Decoder<'_>| {
            Parameter::decode(d).map_err(|e| {
                container::Error::Content(match e {
                    tinycbor::tag::Error::Malformed(error) => {
                        bounded::Error::Content(map::Error::Key(error))
                    }
                    tinycbor::tag::Error::InvalidTag => bounded::Error::Surplus,
                    tinycbor::tag::Error::Content(inner) => {
                        bounded::Error::Content(map::Error::Value(inner))
                    }
                })
            })
        };

        if let Some(len) = d.map_visitor()?.remaining() {
            for _ in 0..len {
                let param = decode_param(d)?;
                if !params.insert(param) {
                    return Err(bounded::Error::Surplus.into());
                }
            }
        } else {
            while d.datatype()? != tinycbor::Type::Break {
                let param = decode_param(d)?;
                if !params.insert(param) {
                    return Err(bounded::Error::Surplus.into());
                }
            }
            d.next().expect("found break").expect("valid break");
        };
        Ok(params)
    }
}
