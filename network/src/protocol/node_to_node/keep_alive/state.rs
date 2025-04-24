use crate::{
    traits::state::{self, State},
    typefu::coproduct::Coprod,
};

use super::message::KeepAlive;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Client;

impl State for Client {
    const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(97);

    type Agency = state::Client;

    type Message = Coprod![KeepAlive, super::message::Done];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Server;

impl State for Server {
    const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

    type Agency = state::Server;

    type Message = Coprod![super::message::Response];
}
