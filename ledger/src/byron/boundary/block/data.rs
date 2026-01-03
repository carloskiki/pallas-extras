use tinycbor_derive::{CborLen, Decode, Encode};
use crate::{byron::block::Difficulty, epoch};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, CborLen)]
pub struct Data {
    pub epoch: epoch::Number,
    pub difficulty: [Difficulty; 1],
}
