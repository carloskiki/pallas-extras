use state::{CanAwait, Idle, Intersect, MustReply};

use crate::{traits::mini_protocol::MiniProtocol, typefu::coproduct::Coproduct};

use super::Coprod;

pub mod message;
pub mod state;

pub type ChainSync = Coprod![Idle,  Intersect, CanAwait, MustReply];

impl Default for ChainSync {
    fn default() -> Self {
        Coproduct::inject(Idle::default())
    }
}

impl MiniProtocol for ChainSync {
    const NUMBER: u16 = 2;
    const READ_BUFFER_SIZE: usize = 200;
}
