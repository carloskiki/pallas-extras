use crate::{epoch, interval, shelley::transaction::Coin};
use sparse_struct::SparseStruct;
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
    PoolPledgeInfluence(interval::Unsigned),
    #[n(10)]
    ExpansionRate(interval::Unit),
    #[n(11)]
    TreasuryGrowthRate(interval::Unit),
    #[n(12)]
    DecentralizationConstant(interval::Unit),
    #[n(13)]
    ExtraEntropy(#[cbor(with = "cbor_util::option::Array<[u8; 32], true>")] Option<[u8; 32]>),
    #[n(14)]
    ProtocolVersion(Version),
    #[n(15)]
    MinimumUtxoValue(Coin),
    #[n(16)]
    MinimumPoolCost(Coin),
}

cbor_util::sparse_struct_impl!(Parameters);
