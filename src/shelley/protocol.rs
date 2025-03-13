use crate::Blake2b224Digest;

use super::{Nonce, RealNumber};

pub struct ProtocolVersion {
    /// TODO: (Major, Minor). Major is in the range [1 .. 9], so we should use an enum.
    pub major: u32,
    pub minor: u32,
}

pub struct ParameterUpdate {
    pub minfee_a: Option<u64>,
    pub minfee_b: Option<u64>,
    pub max_block_body_size: Option<u64>,
    pub max_transaction_size: Option<u64>,
    pub max_block_header_size: Option<u64>,
    pub key_deposit: Option<u64>,
    pub pool_deposit: Option<u64>,
    pub maximum_epoch: Option<u64>,
    pub n_opt: Option<u64>,
    pub pool_pledge_influence: Option<RealNumber>,
    pub expansion_rate: Option<RealNumber>,
    pub treasury_growth_rate: Option<RealNumber>,
    pub decentralization_constant: Option<RealNumber>,
    pub extra_entropy: Option<Nonce>,
    pub protocol_version: Option<ProtocolVersion>,
    pub minimum_utxo_value: Option<u64>,
}

pub struct Update {
    pub proposed: Vec<(Blake2b224Digest, u64)>,
    pub epoch: u64,
}
