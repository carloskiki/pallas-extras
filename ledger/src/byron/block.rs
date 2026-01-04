use crate::byron::Attributes;
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod body;
pub use body::Body;

pub mod consensus;

pub mod data;
pub use data::Data;

pub mod header;
pub use header::Header;

pub mod proof;
pub use proof::Proof;

pub mod signature;
pub use signature::Signature;

pub type Id = crate::crypto::Blake2b256Digest;

pub type Difficulty = u64;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block<'a> {
    pub header: Header<'a>,
    pub body: Body<'a>,
    pub extra: [Attributes<'a>; 1],
}
