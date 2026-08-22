//! Byron era ledger types.
//!
//! This exists for historical validation purposes only. The ledger may assume stricter format rules
//! than the specification, because it is solely meant to view the historical byron era of Cardano
//! mainnet. As such, it uses stricter CBOR decoding rules than what the specification allows. For
//! example, [`Attributes`] assume an empty map.

use tinycbor_derive::{CborLen, Decode, Encode};

pub mod address;
pub use address::Address;

pub mod block;
pub use block::Block;
pub use block::boundary::Block as BoundaryBlock;

pub mod crypto;

pub mod delegation;

pub mod protocol;

pub mod transaction;
pub use transaction::Transaction;

pub mod update;
pub use update::Update;

/// Generic attributes.
///
/// In the Byron era, attributes were liberally attached to many parts of the ledger, but remained
/// unused. This ensures that places where attributes are expected contain the "empty" attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Attributes;
