use crate::traits::{self, message::Message};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct KeepAlive {
    pub cookie: u16,
}

impl Message for KeepAlive {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 0;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = super::state::Server;
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct Response {
    pub cookie: u16,
}

impl Message for Response {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 1;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = super::state::Client;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct Done;

impl Message for Done {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u64 = 2;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = traits::state::Done;
}
