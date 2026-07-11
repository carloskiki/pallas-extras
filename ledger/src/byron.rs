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
