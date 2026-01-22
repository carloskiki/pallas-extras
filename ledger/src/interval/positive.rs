use std::num::NonZeroU64;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(tag(30))]
pub struct Positive {
    pub numerator: NonZeroU64,
    pub denominator: NonZeroU64,
}
