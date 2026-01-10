use crate::byron::Attributes;
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod body;
pub use body::Body;

pub mod data;

pub mod header;
pub use header::Header;

pub mod proof;
pub use proof::Proof;

pub mod signature;
pub use signature::Signature;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block<'a> {
    pub header: Header<'a>,
    pub body: Body<'a>,
    pub extra: [Attributes<'a>; 1],
}
