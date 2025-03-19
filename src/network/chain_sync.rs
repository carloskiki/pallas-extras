use minicbor::{Decode, Encode};

use crate::network;

#[derive(Debug, Encode, Decode)]
#[cbor(flat)]
pub enum ClientMessage {
    #[n(0)]
    Next,
    #[n(4)]
    FindIntersect,
    #[n(7)]
    Done,
}

#[derive(Debug, Encode, Decode)]
#[cbor(flat)]
pub enum ServerMessage {
    #[n(5)]
    IntersectFound,
    #[n(6)]
    IntersectNotFound,
    #[n(2)]
    RollForward,
    #[n(3)]
    RollBackward,
    #[n(1)]
    AwaitReply,
}
