use state::{Busy, Idle, Streaming};

use crate::{traits::mini_protocol::MiniProtocol, typefu::coproduct::Coproduct};

use super::Coprod;

pub mod message;
pub mod state;

pub type BlockFetch = Coprod![Idle, Busy, Streaming];

impl Default for BlockFetch {
    fn default() -> Self {
        Coproduct::inject(Idle::default())
    }
}

impl MiniProtocol for BlockFetch {
    const NUMBER: u16 = 3;
    const READ_BUFFER_SIZE: usize = 100;
}
