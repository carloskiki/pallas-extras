use super::protocol;
use crate::{crypto::Blake2b224Digest, epoch};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Update<'a> {
    pub proposed: Vec<(&'a Blake2b224Digest, protocol::Parameters)>,
    pub epoch: epoch::Number,
}
