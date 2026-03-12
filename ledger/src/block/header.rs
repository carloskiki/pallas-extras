use tinycbor_derive::{CborLen, Decode, Encode};
use crate::{allegra, alonzo, babbage, byron, conway, mary, shelley};

/// Era-independent header.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Header<'a> {
    #[n(0)]
    Boundary(byron::block::boundary::Header<'a>),
    #[n(1)]
    Byron(byron::block::Header<'a>),
    #[n(2)]
    Shelley(shelley::block::Header<'a>),
    #[n(3)]
    Allegra(allegra::block::Header<'a>),
    #[n(4)]
    Mary(mary::block::Header<'a>),
    #[n(5)]
    Alonzo(alonzo::block::Header<'a>),
    #[n(6)]
    Babbage(babbage::block::Header<'a>),
    #[n(7)]
    Conway(conway::block::Header<'a>),
}
