use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Version {
    pub major: Fork,
    #[cbor(with = "tinycbor::num::U8")]
    pub minor: u8,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub enum Fork {
    #[n(1)]
    Byron,
    #[n(2)]
    Shelley,
    #[n(3)]
    Allegra,
    #[n(4)]
    Mary,
    #[n(5)]
    Alonzo,
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
