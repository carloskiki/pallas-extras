pub mod handshake;
pub mod node_to_client;
pub mod node_to_node;

use frunk::{Coproduct, Func};
use minicbor::{Decode, Encode};

pub use node_to_client::NodeToClient;
pub use node_to_node::NodeToNode;

use crate::utilities::{type_maps, CMap, TypeMap};

pub trait Protocol: TypeMap<CMap<type_maps::MiniProtocolMessage>> + Eq + Copy + Sized {
    fn from_number(number: u16) -> Result<Self, UnknownProtocol>;
    fn number(&self) -> u16;
}

impl<S, Tail> Protocol for Coproduct<S, Tail>
where
    S: MiniProtocol + Copy + Eq,
    Tail: Protocol,
{
    fn from_number(number: u16) -> Result<Self, UnknownProtocol> {
        if number == S::NUMBER {
            Ok(Coproduct::Inl(S::default()))
        } else {
            Tail::from_number(number).map(Coproduct::Inr)
        }
    }

    fn number(&self) -> u16 {
        match self {
            Coproduct::Inl(_) => S::NUMBER,
            Coproduct::Inr(tail) => tail.number(),
        }
    }
}

pub trait MiniProtocol: TypeMap<type_maps::StateMessage> + Default {
    const NUMBER: u16;
    const READ_BUFFER_SIZE: usize;
}

// You have a client/server in a given state. You send a message, you end up in a new state.
// To receive a message, you typemap over the message coproduct to generate a
// coproduct of the new states.
//
// You are able to clone clients & servers, so the receiver thread needs to know which message goes
// to which instance.
//
// When receiving a message:
// If the associated state of the message you just received is the same as the one that was sent by
// the message, then keep your receiving spot in the queue. Otherwise, hand your spot to the next
// one in line.

pub trait Message: Encode<()> + for<'a> Decode<'a, ()> {
    const SIZE_LIMIT: usize;
    
    type FromState: State;
    type ToState: State;
}

pub trait State {
    const TIMEOUT: std::time::Duration;

    type Agency: Agency;
    type Message: Message;
}

pub trait Agency {
    const SERVER: bool;
}

enum Client {}
impl Agency for Client {
    const SERVER: bool = false;
}

enum Server {}
impl Agency for Server {
    const SERVER: bool = true;
}

enum MessageToAgency {}

impl<T: Message> Func<T> for MessageToAgency {
    type Output = bool;

    fn call(_: T) -> Self::Output {
        <T::ToState as State>::Agency::SERVER
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnknownProtocol;

impl std::fmt::Display for UnknownProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown protocol number")
    }
}

impl std::error::Error for UnknownProtocol {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnknownMessage;

impl std::fmt::Display for UnknownMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown message")
    }
}

impl std::error::Error for UnknownMessage {}
