use crate::typefu::{
    coproduct::{CNil, Coproduct},
    map::{CMap, HMap, Identity, TypeMap},
};

use super::{mini_protocol::{self, MiniProtocol}, state};

pub trait Protocol: Eq + Copy + Sized
where
    Self: Eq + Copy + Sized,
    CMap<mini_protocol::Message>: TypeMap<Self>,
    HMap<Identity>: TypeMap<Self>,
{
    fn from_number(number: u16) -> Result<Self, UnknownProtocol>;
    fn number(&self) -> u16;
}

impl<S, Tail> Protocol for Coproduct<S, Tail>
where
    CMap<state::Message>: TypeMap<S>,
    S: MiniProtocol + Copy + Eq,
    Tail: Protocol,
    mini_protocol::Message: TypeMap<S>,
    CMap<mini_protocol::Message>: TypeMap<Tail>,
    HMap<Identity>: TypeMap<Tail>,
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

impl Protocol for CNil {
    fn from_number(_: u16) -> Result<Self, UnknownProtocol> {
        Err(UnknownProtocol)
    }

    fn number(&self) -> u16 {
        unreachable!()
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

pub type Message<P> = <CMap<mini_protocol::Message> as TypeMap<P>>::Output;
pub type List<P> = <HMap<Identity> as TypeMap<P>>::Output;
