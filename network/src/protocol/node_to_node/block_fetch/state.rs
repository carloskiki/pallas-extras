use std::time::Duration;

use crate::{traits::state::{Client, Server, State}, typefu::coproduct::Coprod};

use super::message::{NoBlocks, RequestRange, StartBatch};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Idle<const MAINNET: bool>;

impl<const M: bool> State for Idle<M> {
    const TIMEOUT: Duration = Duration::MAX;

    type Agency = Client;
    type Message = Coprod![RequestRange<M>, super::message::Done];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Busy<const MAINNET: bool>;

impl<const M: bool> State for Busy<M> {
    const TIMEOUT: Duration = Duration::from_secs(60);

    type Agency = Server;
    type Message = Coprod![NoBlocks<M>, StartBatch<M>];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Streaming<const MAINNET: bool>;

impl<const M: bool> State for Streaming<M> {
    const TIMEOUT: Duration = Duration::from_secs(60);
    
    type Agency = Server;
    type Message = Coprod![super::message::Block<M>, super::message::Done];
}
