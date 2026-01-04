use tinycbor_derive::{CborLen, Decode, Encode};

pub mod address;
pub use address::Address;

pub mod attributes;
pub use attributes::Attributes;

pub mod block;

pub mod boundary;

pub mod delegation;

pub mod protocol;

pub mod transaction;
pub use transaction::Transaction;

pub mod update;
pub use update::Update;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[allow(clippy::large_enum_variant)]
pub enum Block<'a> {
    #[n(0)]
    Boundary(boundary::Block<'a>),
    #[n(1)]
    Main(block::Block<'a>),
}
