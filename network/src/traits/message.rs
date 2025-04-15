use minicbor::{Decode, Encode};

use crate::typefu::coproduct::{CNil, Coproduct};

use super::state::State;

pub trait Message: Encode<()> + for<'a> Decode<'a, ()> + 'static {
    const SIZE_LIMIT: usize;
    const TAG: u8;
    const ELEMENT_COUNT: u64;

    type ToState: State;
}

impl<M, Tail> Encode<()> for Coproduct<M, Tail>
where
    M: Message,
    Tail: Encode<()>,
{
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        c: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Coproduct::Inl(m) => e.array(M::ELEMENT_COUNT + 1)?.u8(M::TAG)?.encode(m)?.ok(),
            Coproduct::Inr(tail) => tail.encode(e, c),
        }
    }
}

impl Encode<()> for CNil {
    fn encode<W: minicbor::encode::Write>(
        &self,
        _: &mut minicbor::Encoder<W>,
        _: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match *self {}
    }
}

pub(crate) struct TagContext(pub u8);
impl<'a, M, Tail> Decode<'a, TagContext> for Coproduct<M, Tail>
where
    M: Message,
    Tail: Decode<'a, TagContext>,
{
    fn decode(
        d: &mut minicbor::Decoder<'a>,
        TagContext(tag): &mut TagContext,
    ) -> Result<Self, minicbor::decode::Error> {
        Ok(if *tag == M::TAG {
            Coproduct::Inl(d.decode::<M>()?)
        } else {
            Coproduct::Inr(Tail::decode(d, &mut TagContext(*tag))?)
        })
    }
}

impl<C> Decode<'_, C> for CNil {
    fn decode(_: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        Err(minicbor::decode::Error::custom(UnknownMessage))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnknownMessage;

impl std::fmt::Display for UnknownMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown message")
    }
}

impl std::error::Error for UnknownMessage {}

macro_rules! nop_codec {
    ($($name:ty),* $(,)?) => {
        $(
            impl<C> Encode<C> for $name {
                fn encode<W: minicbor::encode::Write>(
                    &self,
                    _: &mut minicbor::Encoder<W>,
                    _: &mut C,
                ) -> Result<(), minicbor::encode::Error<W::Error>> {
                    Ok(())
                }
            }

            impl<C> Decode<'_, C> for $name {
                fn decode(
                    _: &mut minicbor::Decoder<'_>,
                    _: &mut C,
                ) -> Result<Self, minicbor::decode::Error> {
                    Ok(Self)
                }
            }
        )*
    };
}
pub(crate) use nop_codec;
