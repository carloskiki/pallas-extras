use crate::{Coin, byron::Address};
use tinycbor_derive::{CborLen, Decode, Encode};

/// A transaction output.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Output {
    /// The address to which the output belongs.
    pub address: Address,
    /// The amount of coins in the output.
    pub amount: Coin,
}
