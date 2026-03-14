//! The Cardano Ledger
//!
//! The root contains era independent types and utilities such as [`Block`], [`crypto`],
//! [`Unique`], etc. Era dependent types are in their respective modules. Types are defined once in
//! their respective era module, and reused if necessary in newer eras. For example, data for
//! plutus scripts is defined as [`alonzo::script::Data`] and reused in all following eras.

extern crate alloc;

pub mod crypto;
pub mod epoch;
pub mod interval;
pub mod slot;

mod address;
pub use address::Address;

pub mod block;
pub use block::Block;

mod transaction;
pub use transaction::Transaction;

mod unique;
pub use unique::Unique;

mod url;
pub use url::Url;

pub mod allegra;
pub mod alonzo;
pub mod babbage;
pub mod byron;
pub mod conway;
pub mod mary;
pub mod shelley;
