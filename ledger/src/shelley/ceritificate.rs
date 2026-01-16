use crate::{
    crypto::{Blake2b224Digest, Blake2b256Digest}, epoch, shelley::{UnitInterval, address::Account, pool, transaction::Coin}
};

pub enum Certificate<'a> {
    AccountRegistration {
        account: Account<'a>,
    },
    AccountUnregistration {
        account: Account<'a>,
    },
    Delegation {
        account: Account<'a>,
        pool: &'a pool::Id,
    },
    PoolRegistration {
        operator: &'a pool::Id,
        vrf_keyhash: &'a Blake2b256Digest,
        pledge: Coin,
        cost: Coin,
        margin: UnitInterval,
        account: Account<'a>,
        owners: Vec<Account<'a>>,
        relays: Vec<pool::Relay<'a>>,
        pool_metadata: Option<pool::Metadata<'a>>,
    },
    PoolRetirement {
        pool: &'a pool::Id,
        epoch: epoch::Number,
    },
    GenesisDelegation {
        hash: &'a Blake2b224Digest,
        delegate: &'a Blake2b224Digest,
        vrf_keyhash: &'a Blake2b256Digest,
    },
    MoveRewards {
        // TODO
    }
}
