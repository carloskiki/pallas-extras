use crate::{allegra, alonzo, babbage, byron, conway, mary, shelley};
use tinycbor_derive::{CborLen, Decode, Encode};

mod header;
pub use header::Header;

pub type Number = u64;
pub type Size = u32;
pub type Id = crate::crypto::Blake2b256Digest;

/// Era-independent block.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Block<'a> {
    #[n(0)]
    Boundary(byron::BoundaryBlock<'a>),
    #[n(1)]
    Byron(byron::Block<'a>),
    #[n(2)]
    Shelley(shelley::Block<'a>),
    #[n(3)]
    Allegra(allegra::Block<'a>),
    #[n(4)]
    Mary(mary::Block<'a>),
    #[n(5)]
    Alonzo(alonzo::Block<'a>),
    #[n(6)]
    Babbage(babbage::Block<'a>),
    #[n(7)]
    Conway(conway::Block<'a>),
}
