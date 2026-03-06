use tinycbor_derive::{Encode, Decode, CborLen};
use crate::{epoch, slot};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Id {
    pub epoch: epoch::Number,
    pub slot: slot::Number,
}
