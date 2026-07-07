pub use ledger::{crypto::Blake2b224Digest, slot};

pub struct State {
}

pub struct Delegation {
    pub slot: slot::Number,
    pub delegator: Blake2b224Digest,
    pub delegate: Blake2b224Digest,
}
