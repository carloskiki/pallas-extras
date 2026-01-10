//! Types for the Cardano Ledger
//!
//! All types serialize into the Babbage era of the specification, and can be deserialized from any
//! era between and including Shelley and Babbage.

pub mod address;
pub use address::Address;
pub use address::Account;

// TODO: Mary and later
// pub mod asset;
// pub use asset::Asset;

pub mod credential;
pub use credential::Credential;

pub mod unit_interval;
pub use unit_interval::UnitInterval;

// pub mod block;
// pub mod certificate;
// pub mod pool;
pub mod protocol;
// pub mod script;
pub mod transaction;
// pub mod witness;
// pub mod governance;
// pub mod epoch;
// pub mod slot;
// 
// pub use block::Block;
// pub use certificate::Certificate;
// pub use script::Script;
// pub use transaction::Transaction;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Network(pub u8);

impl Network {
    pub const MAIN: Self = Network(1);
    pub const TEST: Self = Network(0);

    pub fn main(&self) -> bool {
        self.0 == 1
    }

    pub fn test(&self) -> bool {
        self.0 == 0
    }

    pub fn unknown(&self) -> bool {
        self.0 > 1
    }
}

