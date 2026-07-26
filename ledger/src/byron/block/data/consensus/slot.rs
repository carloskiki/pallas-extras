use crate::{epoch, slot};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Id {
    pub epoch: epoch::Number,
    pub slot: slot::Number,
}
