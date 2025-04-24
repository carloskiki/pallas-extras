use crate::{traits::mini_protocol::MiniProtocol, HList};
use state::{Client, Server};

pub mod state;
pub mod message;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeepAlive;

impl MiniProtocol for KeepAlive {
    const NUMBER: u16 = 8;

    const READ_BUFFER_SIZE: usize = 1;

    type States = HList![Client, Server];
}
