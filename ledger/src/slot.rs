use crate::epoch;
use tinycbor_derive::{CborLen, Decode, Encode};

pub type Number = u64;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Id {
    pub epoch: epoch::Number,
    pub slot: Number,
}
