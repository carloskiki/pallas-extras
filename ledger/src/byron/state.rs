use crate::{
    Coin, Unique,
    byron::{block, protocol, transaction, update},
    crypto::Blake2b224Digest,
    epoch, slot,
};
use std::collections::HashMap;

pub struct State {
    pub slot: slot::Number,
    pub hash: block::Id,
    pub utxo: HashMap<Input, Output>,
    pub epoch: epoch::Number,
    pub version: protocol::Version,
    pub parameters: protocol::Parameters,
    pub update_candidates: Vec<update::Candidate>,
    // pub application_versions: HashMap<String, _>,
    pub registered_update_proposals: HashMap<update::proposal::Id, update::proposal::Registration>,
    // pub registered_software_proposals: ...
    pub confirmed_proposals: HashMap<update::proposal::Id, slot::Number>,
    pub proposal_votes: HashMap<update::proposal::Id, Unique<Vec<Blake2b224Digest>, true>>,
    pub registered_endorsements: Unique<Vec<Blake2b224Digest>, true>,
    pub proposal_registration_slot: HashMap<update::proposal::Id, slot::Number>,

    pub scheduled_delegations: Vec<Delegation>,
    pub key_epoch_delegations: Unique<Vec<(epoch::Number, Blake2b224Digest)>, true>,

    pub delegations: HashMap<Blake2b224Digest, Blake2b224Digest>, // Should be bimap.
    pub delegation_slots: HashMap<Blake2b224Digest, slot::Number>,
}

pub struct Input {
    pub id: transaction::Id,
    pub index: u32,
}

pub struct Output {
    pub address: Box<[u8]>,
    pub amount: Coin,
}

pub struct Delegation {
    pub slot: slot::Number,
    pub delegator: Blake2b224Digest,
    pub delegate: Blake2b224Digest,
}

pub struct Endorsement {
    pub version: protocol::Version,
    pub key: Blake2b224Digest,
}
