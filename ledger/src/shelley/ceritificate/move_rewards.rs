use tinycbor_derive::{Encode, Decode, CborLen};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct MoveRewards {
    pub source: Source,
    pub target: Option<>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub enum Source {
    #[n(0)]
    Reserves,
    #[n(1)]
    Treasury,
}
