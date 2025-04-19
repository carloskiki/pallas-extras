use std::time::Duration;

use crate::{traits::state::{Client, Server, State}, typefu::coproduct::Coprod};

use super::message::{NoBlocks, RequestRange, StartBatch};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Idle;

impl State for Idle {
    const TIMEOUT: Duration = Duration::MAX;

    type Agency = Client;
    type Message = Coprod![RequestRange, super::message::Done];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Busy;

impl State for Busy {
    const TIMEOUT: Duration = Duration::from_secs(60);

    type Agency = Server;
    type Message = Coprod![NoBlocks, StartBatch];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Streaming;

impl State for Streaming {
    const TIMEOUT: Duration = Duration::from_secs(60);
    
    type Agency = Server;
    type Message = Coprod![super::message::Block, super::message::Done];
}
