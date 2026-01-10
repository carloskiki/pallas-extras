use std::num::NonZeroU64;

use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(tag(30))]
pub struct UnitInterval {
    numerator: u64,
    denominator: NonZeroU64,
}

impl UnitInterval {
    pub fn new(numerator: u64, denominator: NonZeroU64) -> Option<Self> {
        if numerator > denominator.get() {
            return None;
        }
        Some(Self {
            numerator,
            denominator,
        })
    }

    pub fn numerator(&self) -> u64 {
        self.numerator
    }

    pub fn denominator(&self) -> NonZeroU64 {
        self.denominator
    }
}
