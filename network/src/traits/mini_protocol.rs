use std::{any::Any, error::Error};

use minicbor::{Decode, Encode};

use crate::typefu::{
    constructor::Constructor,
    coproduct::{CNil, Coproduct},
    map::{CMap, HMap, Identity, TypeMap},
};

use super::{
    message::{TagContext, UnknownMessage},
    state,
};

pub trait MiniProtocol: Default + 'static {
    const NUMBER: u16;
    const READ_BUFFER_SIZE: usize;

    type States: Default;
}

pub enum MessageMap {}
impl<MP> TypeMap<MP> for MessageMap
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    type Output = <CMap<state::Message> as TypeMap<MP::States>>::Output;
}

pub enum Number {}
impl<MP> TypeMap<MP> for Number
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP>,
{
    type Output = u16;
}

impl<MP> Constructor<MP> for Number
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP>,
{
    fn construct() -> Self::Output {
        MP::NUMBER
    }
}

pub type StatesList<MP> = <HMap<Identity> as TypeMap<<MP as MiniProtocol>::States>>::Output;
pub type Message<MP> = <CMap<state::Message> as TypeMap<<MP as MiniProtocol>::States>>::Output;

/// Encode a coproduct of coproduct of messages
pub(crate) struct EncodeContext;
impl<Head, Tail> Encode<EncodeContext> for Coproduct<Head, Tail>
where
    Head: Encode<()>,
    Tail: Encode<EncodeContext>,
{
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut EncodeContext,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Coproduct::Inl(head) => e.encode(head)?.ok(),
            Coproduct::Inr(tail) => e.encode_with(tail, ctx)?.ok(),
        }
    }
}

impl Encode<EncodeContext> for CNil {
    fn encode<W: minicbor::encode::Write>(
        &self,
        _: &mut minicbor::Encoder<W>,
        _: &mut EncodeContext,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match *self {}
    }
}

pub(crate) struct DecodeContext(pub Option<(u8, bool)>);
impl<'a, Head, Tail> Decode<'a, DecodeContext> for Coproduct<Head, Tail>
where
    Head: Decode<'a, TagContext>,
    Tail: Decode<'a, DecodeContext>,
{
    fn decode(
        d: &mut minicbor::Decoder<'a>,
        ctx: &mut DecodeContext,
    ) -> Result<Self, minicbor::decode::Error> {
        let (tag, indef) = match ctx.0 {
            Some(v) => v,
            None => {
                let indef = d.array()?.is_none();
                let tag = d.u8()?;
                ctx.0 = Some((tag, indef));
                (tag, indef)
            }
        };

        match Head::decode(d, &mut TagContext(tag)) {
            Ok(message) => {
                if indef && d.datatype()? != minicbor::data::Type::Break {
                    return Err(minicbor::decode::Error::message("Expected break"));
                }
                
                Ok(Coproduct::Inl(message))
            },
            Err(e)
                if e.source().is_some_and(|s| {
                    core::any::Any::type_id(s) == core::any::Any::type_id(&UnknownMessage)
                }) => {
                    Tail::decode(d, ctx).map(Coproduct::Inr)
                }
            Err(e) => Err(e),
        }
    }
}
