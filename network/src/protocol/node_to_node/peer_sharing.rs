use crate::{traits::mini_protocol::MiniProtocol, HList};
use state::{Idle, Busy};

pub mod state;
pub mod message;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerSharing;

impl MiniProtocol for PeerSharing {
    const NUMBER: u16 = 10;

    const READ_BUFFER_SIZE: usize = 1;

    type States = HList![Idle, Busy];
}
