use std::time::Duration;

use crate::{traits::state::{Client, Server, State}, typefu::coproduct::Coprod};

use super::message::{SharePeers, ShareRequest};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Idle;

impl State for Idle {
    const TIMEOUT: std::time::Duration = Duration::MAX;

    type Agency = Client;

    type Message = Coprod![ShareRequest, super::message::Done];
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Busy;

impl State for Busy {
    const TIMEOUT: std::time::Duration = Duration::from_secs(60);

    type Agency = Server;

    type Message = Coprod![SharePeers];
}
