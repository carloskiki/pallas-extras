use tinycbor_derive::{Decode, Encode, CborLen};
use crate::{
    Point, Tip,
    traits::{
        self,
        message::Message,
    },
};

use super::state::{CanAwait, Idle, Intersect, MustReply};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct Next;

impl Message for Next {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 0;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = CanAwait;
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct FindIntersect {
    pub points: Vec<Point>,
}

impl Message for FindIntersect {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 4;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Intersect;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct Done;

impl Message for Done {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 7;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = traits::state::Done;
}

mod intersect_found {
    use super::*;
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
    #[cbor(naked)]
    pub struct IntersectFound {
        pub point: Point,
        pub tip: Tip,
    }
}
pub use intersect_found::IntersectFound;


impl Message for IntersectFound {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 5;
    const ELEMENT_COUNT: u64 = 2;

    type ToState = Idle;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct IntersectNotFound {
    pub tip: Tip,
}

impl Message for IntersectNotFound {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 6;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Idle;
}

mod roll_forward {
    use super::*;
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
    #[cbor(naked)]
    pub struct RollForward<'a> {
        pub header: ledger::block::Header<'a>,
        pub tip: Tip,
    }
}
pub use roll_forward::RollForward;


impl Message for RollForward<'_> {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 2;
    const ELEMENT_COUNT: u64 = 2;

    type ToState = Idle;
}

mod roll_backward {
    use super::*;
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
    #[cbor(naked)]
    pub struct RollBackward {
        pub point: Point,
        pub tip: Tip,
    }
}
pub use roll_backward::RollBackward;

impl Message for RollBackward {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 3;
    const ELEMENT_COUNT: u64 = 2;

    type ToState = Idle;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct AwaitReply;

impl Message for AwaitReply {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 1;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = MustReply;
}
