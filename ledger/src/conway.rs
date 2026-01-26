//! Types for the Cardano Ledger
//!
//! All types serialize into the Babbage era of the specification, and can be deserialized from any
//! era between and including Shelley and Babbage.

pub mod asset;
pub use asset::Asset;

pub mod block;
pub use block::Block;

pub mod certificate;
pub use certificate::Certificate;

pub mod governance;

pub mod pool;

pub mod protocol;

pub mod script;
pub use script::Script;

pub mod transaction;
pub use transaction::Transaction;

pub mod url;
pub use url::Url;
// To be able to `duplicate!` `url` from `shelley`.
const URL_SIZE: usize = 128;
