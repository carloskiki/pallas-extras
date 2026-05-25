use tinycbor_derive::{CborLen, Decode, Encode};
use tinycbor::encoded::With;

pub mod data;
pub use data::Data;

pub mod header;
pub use header::Header;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block<'a> {
    pub header: With<'a, Header<'a>>,
    pub body: Vec<&'a crate::crypto::Blake2b224Digest>,
    pub extra: [crate::byron::Attributes<'a>; 1],
}
