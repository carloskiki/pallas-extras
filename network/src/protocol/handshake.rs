use minicbor::{Decode, Encode};
use state::{Confirm, Propose};

use crate::{
    traits::mini_protocol::MiniProtocol,
    typefu::{coproduct::Coprod, hlist::HList},
};

pub mod message;
pub mod state;

pub struct Handshake<VD>(std::marker::PhantomData<VD>);

impl<T> Clone for Handshake<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handshake<T> {}

impl<T> PartialEq for Handshake<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T> Eq for Handshake<T> {}

impl<VD> Default for Handshake<VD>
where
    VD: Encode<()> + for<'a> Decode<'a, ()> + 'static,
{
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<VD> MiniProtocol for Handshake<VD>
where
    VD: Encode<()> + for<'a> Decode<'a, ()> + 'static,
{
    const NUMBER: u16 = 0;
    const READ_BUFFER_SIZE: usize = 1;

    type States = HList![Propose<VD>, Confirm<VD>];
}
