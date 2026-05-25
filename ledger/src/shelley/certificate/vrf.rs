use crate::crypto::vrf::{Hash, Proof};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Vrf<'a> {
    pub output: &'a Hash,
    #[cbor(with = "cbor_util::Bytes<'a, Proof>")]
    pub proof: &'a Proof,
}
