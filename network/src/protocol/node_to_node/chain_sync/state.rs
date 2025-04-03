use std::time::Duration;

use crate::{
    traits::state::{Client, Server, State},
    typefu::coproduct::Coprod,
};

use super::message::{
    self, AwaitReply, FindIntersect, IntersectFound, IntersectNotFound, Next, RollBackward,
    RollForward,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Idle;

impl State for Idle {
    const TIMEOUT: Duration = Duration::from_secs(3673);

    type Agency = Client;
    type Message = Coprod![Next, FindIntersect, message::Done];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Intersect;

impl State for Intersect {
    const TIMEOUT: Duration = Duration::from_secs(10);

    type Agency = Server;

    type Message = Coprod![IntersectFound, IntersectNotFound];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanAwait;

impl State for CanAwait {
    const TIMEOUT: Duration = Duration::from_secs(10);

    type Agency = Server;

    type Message = Coprod![AwaitReply, RollForward, RollBackward];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MustReply;

impl State for MustReply {
    // According to the spec, this should be random between 135 and 269 seconds - we use the
    // lower bound here.
    const TIMEOUT: Duration = Duration::from_secs(135);

    type Agency = Server;

    type Message = Coprod![RollForward, RollBackward];
}
