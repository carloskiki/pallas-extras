use tinycbor_derive::{Encode, Decode, CborLen};
use crate::{shelley::protocol, crypto::Blake2b224Digest, epoch};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Update<'a> {
    pub proposed: Vec<(&'a Blake2b224Digest, protocol::Parameters)>,
    pub epoch: epoch::Number,
}
