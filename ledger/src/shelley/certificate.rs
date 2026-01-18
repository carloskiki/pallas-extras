use crate::{
    crypto::{Blake2b224Digest, Blake2b256Digest},
    epoch,
    shelley::{Credential, UnitInterval, address::Account, pool, transaction::Coin},
};
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod move_rewards;
pub use move_rewards::MoveRewards;

// TODO: move this to its own crate.
pub mod vrf;
pub use vrf::Vrf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub enum Certificate<'a> {
    #[n(0)]
    AccountRegistration { account: Credential<'a> },
    #[n(1)]
    AccountUnregistration { account: Credential<'a> },
    #[n(2)]
    Delegation {
        account: Credential<'a>,
        pool: &'a pool::Id,
    },
    #[n(3)]
    PoolRegistration {
        operator: &'a pool::Id,
        vrf_keyhash: &'a Blake2b256Digest,
        pledge: Coin,
        cost: Coin,
        margin: UnitInterval,
        account: Account<'a>,
        owners: Vec<&'a Blake2b224Digest>,
        relays: Vec<pool::Relay<'a>>,
        pool_metadata: Option<pool::Metadata<'a>>,
    },
    #[n(4)]
    PoolRetirement {
        pool: &'a pool::Id,
        epoch: epoch::Number,
    },
    #[n(5)]
    GenesisDelegation {
        hash: &'a Blake2b224Digest,
        delegate: &'a Blake2b224Digest,
        vrf_keyhash: &'a Blake2b256Digest,
    },
    #[n(6)]
    MoveRewards(MoveRewards<'a>),
}
