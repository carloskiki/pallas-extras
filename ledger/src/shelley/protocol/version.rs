use tinycbor_derive::{CborLen, Decode, Encode};

pub mod major;
pub use major::Major;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Version {
    pub major: Major,
    #[cbor(with = "tinycbor::num::U8")]
    pub minor: u8,
}

