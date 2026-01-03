use tinycbor_derive::{CborLen, Decode, Encode};

pub mod software;
pub use software::Software;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    #[cbor(with = "tinycbor::num::U8")]
    pub patch: u8,
}
