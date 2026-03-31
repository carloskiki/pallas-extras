use crate::{agency::Server, node_to_node::chain_sync::reply::{RollBackward, RollForward}};
use tinycbor_derive::{CborLen, Decode, Encode};

crate::state! {
    CanAwait {
        size_limit: u16::MAX as usize,
        timeout: std::time::Duration::from_secs(10),
        agency: Server,
        message: [AwaitReply, RollForward<'static>, RollBackward]
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct AwaitReply;

impl crate::Message for AwaitReply {
    const TAG: u64 = 1;

    type ToState = super::MustReply;
}
