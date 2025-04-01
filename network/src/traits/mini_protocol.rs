use crate::typefu::{constructor::Constructor, map::{CMap, TypeMap}};

use super::state;

pub trait MiniProtocol: Default
where
    CMap<state::Message>: TypeMap<Self>,
{
    const NUMBER: u16;
    const READ_BUFFER_SIZE: usize;
}

pub enum Message {}
impl<MP> TypeMap<MP> for Message
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP>,
{
    type Output = <CMap<state::Message> as TypeMap<MP>>::Output;
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
