crate::state! {
    MustReply {
        size_limit: u16::MAX as usize,
        // According to the spec, this should be random between 135 and 269 seconds - we use the
        // lower bound here.
        timeout: std::time::Duration::from_secs(135),
        agency: crate::agency::Server,
        message: [RollForward<'static>, RollBackward]
    }
}

mod roll_forward {
    use tinycbor_derive::{Decode, Encode, CborLen};
    
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
    #[cbor(naked)]
    pub struct RollForward<'a> {
        pub header: ledger::block::Header<'a>,
        pub tip: crate::Tip,
    }
}
pub use roll_forward::RollForward;


impl crate::Message for RollForward<'_> {
    const TAG: u64 = 2;

    type ToState = super::Idle;
}

mod roll_backward {
    use tinycbor_derive::{Decode, Encode, CborLen};
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
    #[cbor(naked)]
    pub struct RollBackward {
        pub point: crate::Point,
        pub tip: crate::Tip,
    }
}
pub use roll_backward::RollBackward;

impl crate::Message for RollBackward {
    const TAG: u64 = 3;

    type ToState = super::Idle;
}
