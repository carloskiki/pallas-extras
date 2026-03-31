use crate::{Point, agency::Client, message::Done};
use tinycbor_derive::{CborLen, Decode, Encode};

crate::state! {
    Idle {
        size_limit: u16::MAX as usize,
        timeout: std::time::Duration::from_secs(3673),
        agency: Client,
        message: [Next, FindIntersect, Done<7>]
    }
}

impl crate::state::InitialState for Idle {
    const PROTOCOL_ID: u16 = 7;
    const INGRESS_BUFFER_SIZE: usize = 200;
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen,
)]
#[cbor(naked)]
pub struct Next;

impl crate::Message for Next {
    const TAG: u64 = 0;

    type ToState = super::CanAwait;
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct FindIntersect {
    pub points: Vec<Point>,
}

impl crate::Message for FindIntersect {
    const TAG: u64 = 4;

    type ToState = super::Intersect;
}
