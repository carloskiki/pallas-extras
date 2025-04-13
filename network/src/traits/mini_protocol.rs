use crate::typefu::{
    constructor::Constructor,
    map::{CMap, HMap, Identity, TypeMap},
};

use super::state;

pub trait MiniProtocol: Default {
    const NUMBER: u16;
    const READ_BUFFER_SIZE: usize;

    type States:  Default;
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
