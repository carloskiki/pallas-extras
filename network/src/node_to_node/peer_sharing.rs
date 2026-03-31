use crate::{agency::Server, state};
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod idle;
pub use idle::Idle;

mod share;
pub use share::Share;

state! {
    Busy {
        size_limit: 5760,
        timeout: std::time::Duration::from_secs(60),
        agency: Server,
        message: [Share]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct Request {
    #[cbor(with = "tinycbor::num::U8")]
    pub amount: u8,
}

impl crate::Message for Request {
    const TAG: u64 = 0;

    type ToState = Busy;
}
