use crate::interval;
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod model;
pub use model::Models;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Costs {
    memory: interval::Positive,
    steps: interval::Positive,
}
