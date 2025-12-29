use tinycbor_derive::{CborLen, Decode, Encode};

use crate::byron::Address;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Output {
    pub address: Address,
    pub amount: u64,
}
