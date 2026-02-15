use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Index {
    pub kind: Kind,
    pub index: u64,
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
    #[n(4)]
    Vote,
    #[n(5)]
    Propose,
}
