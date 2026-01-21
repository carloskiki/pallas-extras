use crate::{allegra, byron, mary, shelley};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Block<'a> {
    #[n(0)]
    Boundary(byron::BoundaryBlock<'a>),
    // Boxed because large size
    #[n(1)]
    Byron(Box<byron::Block<'a>>),
    #[n(2)]
    Shelley(shelley::Block<'a>),
    #[n(3)]
    Allegra(allegra::Block<'a>),
    #[n(4)]
    Mary(mary::Block<'a>),
}
