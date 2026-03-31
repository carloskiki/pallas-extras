use tinycbor_derive::{CborLen, Decode, Encode};

pub mod client;
pub use client::Client;

crate::state! {
    Server {
        size_limit: u16::MAX as usize,
        timeout: std::time::Duration::from_secs(60),
        agency: crate::agency::Server,
        message: [Response]
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct KeepAlive {
    pub cookie: u16,
}

impl crate::Message for KeepAlive {
    const TAG: u64 = 0;

    type ToState = Server;
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct Response {
    pub cookie: u16,
}

impl crate::Message for Response {
    const TAG: u64 = 1;

    type ToState = Client;
}
