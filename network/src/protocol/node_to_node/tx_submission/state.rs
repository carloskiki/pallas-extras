use std::time::Duration;

use crate::{traits::state::{Client, Server, State}, typefu::coproduct::Coprod};

use super::message::{ReplyTransactionIds, ReplyTransactions, RequestTransactionIds, RequestTransactions};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Init;

impl State for Init {
    const TIMEOUT: std::time::Duration = Duration::MAX;

    type Agency = Client;

    type Message = Coprod![super::message::Init];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Idle;

impl State for Idle {
    const TIMEOUT: Duration = Duration::MAX;

    type Agency = Server;

    type Message = Coprod![RequestTransactions, RequestTransactionIds<false>, RequestTransactionIds<true>];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Transactions;

impl State for Transactions {
    const TIMEOUT: Duration = Duration::from_secs(10);

    type Agency = Client;

    type Message = Coprod![ReplyTransactions];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionIds<const BLOCKING: bool>;

impl State for TransactionIds<false> {
    const TIMEOUT: Duration = Duration::from_secs(10);

    type Agency = Client;

    type Message = Coprod![ReplyTransactionIds];
}

impl State for TransactionIds<true> {
    const TIMEOUT: Duration = Duration::MAX;

    type Agency = Client;

    type Message = Coprod![ReplyTransactionIds, super::message::Done];
}
