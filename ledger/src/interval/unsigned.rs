use core::num::NonZeroU64;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(tag(30))]
pub struct Unsigned {
    pub numerator: u64,
    pub denominator: NonZeroU64,
}
