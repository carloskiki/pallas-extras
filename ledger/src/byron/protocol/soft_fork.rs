use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Rule {
    pub initial_threshold: u64, // TODO: LovelacePortion newtype...
    pub minimum_threshold: u64,
    pub decrement: u64,
}
