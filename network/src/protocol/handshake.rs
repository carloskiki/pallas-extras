use minicbor::{Decode, Encode};
use state::{Confirm, Propose};

use crate::{
    traits::mini_protocol::MiniProtocol,
    typefu::coproduct::{Coprod, Coproduct},
};

pub mod message;
pub mod state;

pub type Handshake<VD> = Coprod![Propose<VD>, Confirm<VD>];

impl<VD> Default for Handshake<VD>
where
    VD: Encode<()> + for<'a> Decode<'a, ()> + 'static,
{
    fn default() -> Self {
        Coproduct::inject(Propose(Default::default()))
    }
}

impl<VD> MiniProtocol for Handshake<VD>
where
    VD: Encode<()> + for<'a> Decode<'a, ()> + 'static,
{
    const NUMBER: u16 = 0;
    const READ_BUFFER_SIZE: usize = 1;
}
