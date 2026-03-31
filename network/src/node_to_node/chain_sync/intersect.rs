use tinycbor_derive::{Decode, Encode, CborLen};

crate::state! {
    Intersect {
        size_limit: u16::MAX as usize,
        timeout: std::time::Duration::from_secs(10),
        agency: Server,
        message: [Found, NotFound]
    }
}

mod found {
    use tinycbor_derive::{Decode, Encode, CborLen};
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
    #[cbor(naked)]
    pub struct Found {
        pub point: crate::Point,
        pub tip: crate::Tip,
    }
}
pub use found::Found;

use crate::agency::Server;


impl crate::Message for Found {
    const TAG: u64 = 5;

    type ToState = super::Idle;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct NotFound {
    pub tip: crate::Tip,
}

impl crate::Message for NotFound {
    const TAG: u64 = 6;

    type ToState = super::Idle;
}
