use state::{Busy, Idle, Streaming};

use crate::{HList, traits::mini_protocol::MiniProtocol};

pub mod message;
pub mod state;

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub struct BlockFetch<const MAINNET: bool>;

impl<const M: bool> MiniProtocol for BlockFetch<M> {
    const NUMBER: u16 = 3;
    const READ_BUFFER_SIZE: usize = 100;

    type States = HList![Idle<M>, Busy<M>, Streaming<M>];
}
