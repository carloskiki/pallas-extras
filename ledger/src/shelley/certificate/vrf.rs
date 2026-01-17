use tinycbor_derive::{Decode, Encode, CborLen};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Vrf<'a> {
    // TODO: this is `bytes` in the cddl specc, but should always be 64 bytes.
    pub output: &'a [u8; 64],
    pub proof: &'a [u8; 80],
}
