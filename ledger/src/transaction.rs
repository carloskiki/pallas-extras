use crate::crypto::Blake2b256Digest;

use super::{certificate, protocol, witness, address::{Address, StakeAddress}};

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
    pub ttl: Option<u64>,
    pub certificates: Vec<certificate::Certificate>,
    pub withdrawals: Vec<(StakeAddress<false>, u64)>,
    pub update: Option<protocol::Update>,
    pub metadata_hash: Option<Blake2b256Digest>,
}

pub struct Input {
    pub id: Blake2b256Digest,
    pub index: u16,
}

pub struct Output {
    pub address: Address<false>,
    pub amount: u64,
}
