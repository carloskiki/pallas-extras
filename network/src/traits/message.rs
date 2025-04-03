use minicbor::{Decode, Encode};

use crate::typefu::coproduct::{CNil, Coproduct};

use super::state::State;

pub trait Message: Encode<()> + for<'a> Decode<'a, ()> + 'static {
    const SIZE_LIMIT: usize;
    const TAG: u8;
    const ELEMENT_COUNT: u64;

    type ToState: State;
}

impl<C, M, Tail> Encode<C> for Coproduct<M, Tail>
where
    M: Message,
    Tail: Encode<C>,
{
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Coproduct::Inl(m) => e.array(M::ELEMENT_COUNT + 1)?.u8(M::TAG)?.encode(m)?.ok(),
            Coproduct::Inr(tail) => tail.encode(e, ctx),
        }
    }
}

impl<C> Encode<C> for CNil {
    fn encode<W: minicbor::encode::Write>(
        &self,
        _: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        unreachable!()
    }
}

impl<'a, C, M, Tail> Decode<'a, C> for Coproduct<M, Tail>
where
    M: Message,
    Tail: Decode<'a, C>,
{
    fn decode(d: &mut minicbor::Decoder<'a>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        // TODO: this should be done once instead of for every type in the coproduct
        let mut probe = d.probe();
        probe.array()?;
        
        if probe.u8()? == M::TAG {
            let array_len = M::ELEMENT_COUNT + 1;
            let message: M;
            
            match d.array()? {
                Some(l) if l == array_len => message = d.decode()?,
                None => {
                    message = d.decode()?;
                    if d.datatype()? != minicbor::data::Type::Break {
                        return Err(minicbor::decode::Error::message("expected break"));
                    }
                    d.skip()?;
                },
                Some(_) => return Err(minicbor::decode::Error::message("unexpected array length")),
            }
            
            Ok(Coproduct::Inl(message))
        } else {
            Ok(Coproduct::Inr(Tail::decode(d, ctx)?))
        }
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
