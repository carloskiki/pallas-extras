use std::net::{Ipv4Addr, Ipv6Addr};

use either::Either;

use crate::{Blake2b224Digest, Blake2b256Digest};

use super::{witness, protocol, PaymentCredential, RealNumber};

// TODO: temporary fix, decide what to do later. Options:
// - Use a program wide constant for which network we are on (mainnet, preview, preprod).
// - Add the network const everywhere.
type Address = super::Address<true>;
type StakeAddress = super::StakeAddress<true>;

pub struct Transaction {
    pub body: Body,
    pub witness_set: witness::Set,
    pub metadata: Vec<(u64, Metadatum)>,
}

pub enum Metadatum {
    Integer(i64),
    Bytes(Vec<u8>),
    Text(String),
    Array(Vec<Metadatum>),
    Map(Vec<(Metadatum, Metadatum)>),
}

pub struct Body {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub fee: u64,
    pub ttl: u64,
    pub certificates: Vec<Certificate>,
    pub withdrawals: Vec<(StakeAddress, u64)>,
    pub update: Option<protocol::Update>,
    pub metadata_hash: Option<Blake2b256Digest>,
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
    PoolRetirement {
        pool_keyhash: Blake2b224Digest,
        epoch: u64,
    },
    GenesisKeyDelegation {
        genesis_hash: Blake2b224Digest,
        genesis_delegate_hash: Blake2b224Digest,
        vrf_keyhash: Blake2b256Digest,
    },
    MoveRewards {
        from: AccountingPot,
        to: Either<Vec<(StakeAddress, u64)>, u64>,
    },
}

pub struct PoolParameters {
    pub operator: Blake2b224Digest,
    pub vrf_keyhash: Blake2b256Digest,
    pub pledge: u64,
    pub cost: u64,
    pub margin: RealNumber,
    pub reward_account: StakeAddress,
    pub owners: Vec<Blake2b224Digest>,
    pub relays: Vec<Relay>,
    pub metadata: Option<PoolMetadata>,
}

// TODO: what type to use for dns_name?
pub enum Relay {
    HostAddress {
        port: Option<u16>,
        ipv4: Option<Ipv4Addr>,
        ipv6: Option<Ipv6Addr>,
    },
    HostName {
        port: Option<u16>,
        dns_name: String,
    },
    MultiHostName {
        dns_name: String,
    },
}

pub struct PoolMetadata {
    pub url: String,
    pub metadata_hash: Blake2b256Digest,
}

pub enum AccountingPot {
    Reserve,
    Treasury,
}

