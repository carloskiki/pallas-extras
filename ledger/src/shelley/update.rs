use tinycbor_derive::{Encode, Decode, CborLen};
use crate::{crypto::Blake2b224Digest, epoch};
use super::protocol;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Update<'a> {
    pub proposed: Vec<(&'a Blake2b224Digest, protocol::Parameters)>,
    pub epoch: epoch::Number,
}
