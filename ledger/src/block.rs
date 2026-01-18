use crate::{byron, shelley};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Block<'a> {
    #[n(0)]
    Boundary(byron::BoundaryBlock<'a>),
    #[n(1)]
    Byron(byron::Block<'a>),
    #[n(2)]
    Shelley(shelley::Block<'a>),
}
