use crate::typefu::{
    coproduct::{CNil, Coproduct},
    map::{HMap, Identity, TypeMap},
};
use super::mini_protocol::MiniProtocol;

pub trait Protocol: Eq + Copy + Sized + Send + 'static {
    fn from_number(number: u16) -> Option<Self>;
    fn number(&self) -> u16;
}

impl<S, Tail> Protocol for Coproduct<S, Tail>
where
    Tail: Protocol,
    HMap<Identity>: TypeMap<Tail>,
    S: MiniProtocol + Copy + Eq + Send,
{
    fn from_number(number: u16) -> Option<Self> {
        Coproduct::<S, CNil>::from_number(number)
            .map(Coproduct::Inl)
            .or_else(|| Tail::from_number(number).map(Coproduct::Inr))
    }

    fn number(&self) -> u16 {
        match self {
            Coproduct::Inl(_) => S::NUMBER,
            Coproduct::Inr(tail) => tail.number(),
        }
    }
}

impl<S: MiniProtocol + Copy + Eq + Send> Protocol for Coproduct<S, CNil> {
    fn from_number(number: u16) -> Option<Self> {
        if number == S::NUMBER {
            Some(Coproduct::Inl(S::default()))
        } else {
            None
        }
    }

    fn number(&self) -> u16 {
        S::NUMBER
    }
}

pub(crate) type List<P> = <HMap<Identity> as TypeMap<P>>::Output;
