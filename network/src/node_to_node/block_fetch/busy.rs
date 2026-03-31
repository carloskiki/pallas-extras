use tinycbor_derive::{CborLen, Decode, Encode};

use crate::agency::Server;

crate::state! {
    Busy {
        size_limit: u16::MAX as usize,
        timeout: std::time::Duration::from_secs(60),
        agency: Server,
        message: [NoBlocks, StartBatch]
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct NoBlocks;

impl crate::Message for NoBlocks {
    const TAG: u64 = 3;

    type ToState = super::Idle;
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct StartBatch;

impl crate::Message for StartBatch {
    const TAG: u64 = 2;

    type ToState = super::Streaming;
}
