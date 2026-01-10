use tinycbor_derive::{CborLen, Decode, Encode};

pub mod boundary;

pub mod main;

pub type Id = crate::crypto::Blake2b256Digest;

pub type Difficulty = u64;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[allow(clippy::large_enum_variant)]
pub enum Block<'a> {
    #[n(0)]
    Boundary(boundary::Block<'a>),
    #[n(1)]
    Main(main::Block<'a>),
}
