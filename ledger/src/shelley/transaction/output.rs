pub use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Output<'a> {
    pub address: crate::Address<'a>,
    pub amount: super::Coin,
}
