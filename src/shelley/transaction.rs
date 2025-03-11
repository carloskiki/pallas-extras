use crate::{Blake2b224Digest, Blake2b256Digest};

use super::PaymentCredential;

// TODO: temporary fix, decide what to do later. Options:
// - Use a program wide constant for which network we are on (mainnet, preview, preprod).
// - Add the network const everywhere.
type Address = super::PaymentAddress<true>;

pub struct Body {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub fee: u64,
    pub ttl: u64,
    pub certificates: Vec<Certificate>,
}

pub struct Input {
    pub id: Blake2b256Digest,
    pub index: u16,
}

pub struct Output {
    pub address: Address,
    pub amount: u64,
}

pub enum Certificate {
    StakeRegistration {
        stake_credential: PaymentCredential,
    },
    StakeDeregistration {
        stake_credential: PaymentCredential,
    },
    StakeDelegation {
        stake_credential: PaymentCredential,
        pool_keyhash: Blake2b224Digest,
    },
    PoolRegistration {
        parameters: PoolParameters,
    },
    PoolRetirement,
    GenesisKeyDelegation,
    MoveRewards,
}

pub struct PoolParameters {
    pub operator: Blake2b224Digest,
    pub vrf_keyhash: Blake2b256Digest,
    pub pledge: u64,
    pub cost: u64,
    pub margin: RealNumber,
    // TODO: Split address in two.
}

pub struct RealNumber {
    pub numerator: u64,
    pub denominator: u64,
}
