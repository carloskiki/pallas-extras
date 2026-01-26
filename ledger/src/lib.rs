//! Types for the Cardano Ledger
//!
//! All types serialize into the Babbage era of the specification, and can be deserialized from any
//! era between and including Shelley and Babbage.

extern crate alloc;

pub mod crypto;
pub mod slot;
pub mod epoch;
pub mod interval;

pub mod address;
pub use address::Address;

pub mod block;
pub use block::Block;

pub mod conway;
pub mod babbage;
pub mod alonzo;
pub mod mary;
pub mod allegra;
pub mod shelley;
pub mod byron;
