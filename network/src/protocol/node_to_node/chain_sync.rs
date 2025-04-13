use state::{CanAwait, Idle, Intersect, MustReply};

use crate::{traits::mini_protocol::MiniProtocol, typefu::hlist::HList};

pub mod message;
pub mod state;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainSync;

impl MiniProtocol for ChainSync {
    const NUMBER: u16 = 2;
    const READ_BUFFER_SIZE: usize = 200;

    type States = HList![Idle,  Intersect, CanAwait, MustReply];
}
