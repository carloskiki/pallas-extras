use state::{Busy, Idle, Streaming};

use crate::{traits::mini_protocol::MiniProtocol, HList};

pub mod message;
pub mod state;

#[derive(Debug, Default)]
pub struct BlockFetch;

impl MiniProtocol for BlockFetch {
    const NUMBER: u16 = 3;
    const READ_BUFFER_SIZE: usize = 100;

    type States = HList![Idle, Busy, Streaming];
}
