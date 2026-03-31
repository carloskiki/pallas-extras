use tinycbor_derive::{CborLen, Decode, Encode};

crate::state! {
    Streaming {
        size_limit: 2_500_000,
        timeout: std::time::Duration::from_secs(60),
        agency: crate::agency::Server,
        message: [Block<'static>, BatchDone]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct Block<'a>(#[cbor(with = "tinycbor::Encoded<ledger::Block<'a>>")] pub ledger::Block<'a>);

impl<'a> crate::Message for Block<'a> {
    const TAG: u64 = 4;

    type ToState = super::Streaming;
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct BatchDone;

impl crate::Message for BatchDone {
    const TAG: u64 = 5;

    type ToState = super::Idle;
}
