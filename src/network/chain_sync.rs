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

enum State {
    Idle,
    Intersect,
    Next { awaiting: bool },
    Done,
}

pub trait StateMachine {
    type Error: std::error::Error;
    type ClientMessage;
    type ServerMessage;

    fn update_server(
        &mut self,
        message: Self::ClientMessage,
    ) -> network::Result<Self::ServerMessage, Self::Error>;

    fn update_client(
        &mut self,
        message: Self::ServerMessage,
    ) -> network::Result<Self::ClientMessage, Self::Error>;
}
