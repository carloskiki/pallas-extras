use crate::agency::Client;
use crate::message::Done;

crate::state! {
    Idle {
        size_limit: u16::MAX as usize,
        timeout: std::time::Duration::MAX,
        agency: Client,
        message: [Done<1>, RequestRange]
    }
}

impl crate::state::InitialState for Idle {
    const PROTOCOL_ID: u16 = 3;
    const INGRESS_BUFFER_SIZE: usize = 100;
}

mod request_range {
    use crate::Point;
    use tinycbor_derive::{CborLen, Decode, Encode};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
    #[cbor(naked)]
    pub struct RequestRange {
        pub start: Point,
        pub end: Point,
    }
}
pub use request_range::RequestRange;

impl crate::Message for RequestRange {
    const TAG: u64 = 0;

    type ToState = super::Busy;
}
