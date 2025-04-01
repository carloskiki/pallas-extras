use minicbor::{Decode, Encode};

use super::state::State;

pub trait Message: Encode<()> + for<'a> Decode<'a, ()> + 'static {
    const SIZE_LIMIT: usize;
    const TAG: u8;
    
    type FromState: State;
    type ToState: State;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnknownMessage;

impl std::fmt::Display for UnknownMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown message")
    }
}

impl std::error::Error for UnknownMessage {}
