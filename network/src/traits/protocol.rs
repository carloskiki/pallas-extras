use crate::typefu::{
    coproduct::{CNil, Coproduct},
    map::{CMap, HMap, Identity, TypeMap},
};

use super::mini_protocol::{self, MiniProtocol};

pub trait Protocol: Eq + Copy + Sized + Send + 'static
{
    fn from_number(number: u16) -> Result<Self, UnknownProtocol>;
    fn number(&self) -> u16;
}

impl<S, Tail> Protocol for Coproduct<S, Tail>
where
    Tail: Protocol,
    HMap<Identity>: TypeMap<Tail>,
    S: MiniProtocol + Copy + Eq + Send,
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

pub(crate) type List<P> = <HMap<Identity> as TypeMap<P>>::Output;
