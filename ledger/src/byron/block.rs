use crate::byron::Attributes;
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod body;
pub use body::Body;

pub mod boundary;

pub mod data;

mod header;
pub use header::Header;

mod proof;
pub use proof::Proof;

mod signature;
pub use signature::Signature;

pub type Difficulty = u64;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block<'a> {
    pub header: Box<Header<'a>>,
    pub body: Body<'a>,
    pub extra: [Attributes; 1],
}
