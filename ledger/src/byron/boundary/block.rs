use tinycbor_derive::{CborLen, Decode, Encode};

pub mod data;
pub use data::Data;

pub mod header;
pub use header::Header;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block {
    pub header: Header,
    pub body: Vec<crate::crypto::Blake2b224Digest>,
    pub extra: [crate::byron::Attributes; 1],
}
