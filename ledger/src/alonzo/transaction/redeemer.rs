use crate::alonzo::script::{Data, execution};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Redeemer {
    pub kind: Kind,
    pub index: u64,
    pub data: Data,
    pub execution_units: execution::Units,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub enum Kind {
    #[n(0)]
    Spend,
    #[n(1)]
    Mint,
    #[n(2)]
    Certify,
    #[n(3)]
    Reward,
}
