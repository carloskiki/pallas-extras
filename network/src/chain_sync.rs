use minicbor::{Decode, Encode};

use super::{Point, Tip};

#[derive(Debug, Encode, Decode)]
#[cbor(flat)]
pub enum ClientMessage {
    #[n(0)]
    Next,
    #[n(4)]
    FindIntersect {
        #[n(0)]
        #[cbor(with = "cbor_util::boxed_slice")]
        points: Box<[Point]>,
    },
    #[n(7)]
    Done,
}

#[derive(Debug, Encode, Decode)]
#[cbor(flat)]
pub enum ServerMessage {
    #[n(5)]
    IntersectFound {
        #[n(0)]
        point: Point,
        #[n(1)]
        tip: Tip,
    },
    #[n(6)]
    IntersectNotFound {
        #[n(0)]
        tip: Tip,
    },
    #[n(2)]
    RollForward {
        #[n(0)]
        header: Box<ledger::block::Header>,
        #[n(1)]
        tip: Tip,
    },
    #[n(3)]
    RollBackward {
        #[n(0)]
        point: Point,
        #[n(1)]
        tip: Tip,
    },
    #[n(1)]
    AwaitReply,
}
