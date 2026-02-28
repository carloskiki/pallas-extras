use super::protocol;
use crate::{Unique, crypto::Blake2b224Digest, epoch};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Update<'a> {
    pub proposed: Unique<Vec<(&'a Blake2b224Digest, protocol::Parameters)>, false>,
    pub epoch: epoch::Number,
}
