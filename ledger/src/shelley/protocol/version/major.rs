use crate::era::Era;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(tag_only)]
pub enum Major {
    #[n(2)]
    Shelley,
    #[n(3)]
    Allegra,
    // #[n(4)]
    // Mary,
    // #[n(5)]
    // Alonzo,
    // #[n(6)]
    // Lobster,
    // #[n(7)]
    // Vasil,
    // #[n(8)]
    // Valentine,
    // #[n(9)]
    // Chang,
    // #[n(10)]
    // Plomin,
    // #[n(11)]
    // Next,
}

impl Major {
    pub fn era(self) -> Era {
        match self {
            Major::Shelley => Era::Shelley,
            Major::Allegra => Era::Allegra,
        }
    }
}
