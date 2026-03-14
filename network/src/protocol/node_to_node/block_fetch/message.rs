use crate::{
    Point,
    traits::{message::Message, state},
};
use tinycbor_derive::{CborLen, Decode, Encode};

use super::state::{Busy, Idle, Streaming};

mod request_range {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
    #[cbor(naked)]
    pub struct RequestRange {
        pub start: Point,
        pub end: Point,
    }
}
pub use request_range::RequestRange;

impl Message for RequestRange {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 0;
    const ELEMENT_COUNT: u64 = 2;

    type ToState = Busy;
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct NoBlocks;

impl Message for NoBlocks {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 3;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = Idle;
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct StartBatch;

impl Message for StartBatch {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 2;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = Streaming;
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct Block<'a>(#[cbor(with = "tinycbor::Encoded<ledger::Block<'a>>")] pub ledger::Block<'a>);

impl<'a> Message for Block<'a> {
    const SIZE_LIMIT: usize = 2_500_000;
    const TAG: u64 = 4;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Streaming;
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct BatchDone;

impl Message for BatchDone {
    // In the spec, this is 2_500_000, but that's absurdly large for nothing
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 5;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = Idle;
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct Done;

impl Message for Done {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 1;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = state::Done;
}
