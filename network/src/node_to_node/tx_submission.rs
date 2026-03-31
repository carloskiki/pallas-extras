use std::time::Duration;
use crate::{Message, State, agency::Client};
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod request;
pub mod reply;
pub mod blocking;

pub mod idle;
pub use idle::Idle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct Init;

impl State for Init {
    const SIZE_LIMIT: usize = 5760;
    const TIMEOUT: Duration = Duration::MAX;

    type Agency = Client;
    type Message = crate::message::Single<Client, Self>;
}

impl crate::state::InitialState for Init {
    const PROTOCOL_ID: u16 = 4;
    const INGRESS_BUFFER_SIZE: usize = 1;
}

impl Message for Init {
    const TAG: u64 = 6;

    type ToState = Idle;
}


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Transactions;

impl State for Transactions {
    const SIZE_LIMIT: usize = 2_500_000;
    const TIMEOUT: Duration = Duration::from_secs(10);

    type Agency = Client;
    type Message = crate::message::Single<Client, reply::Transactions<'static>>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionIds<const BLOCKING: bool>;

impl State for TransactionIds<false> {
    const SIZE_LIMIT: usize = 2_500_000;
    const TIMEOUT: Duration = Duration::from_secs(10);

    type Agency = Client;
    type Message = crate::message::Single<Client, reply::Ids<'static>>;
}

impl State for TransactionIds<true> {
    const SIZE_LIMIT: usize = 2_500_000;
    const TIMEOUT: Duration = Duration::MAX;

    type Agency = Client;

    type Message = blocking::Message;
}
