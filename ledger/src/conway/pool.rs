use std::net::{Ipv4Addr, Ipv6Addr};
use tinycbor_derive::{CborLen, Decode, Encode};

use crate::{conway::url::Url, crypto::Blake2b256Digest};

pub mod metadata;
pub use metadata::Metadata;

pub mod relay;
pub use relay::Relay;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct VotingThresholds {
    #[n(0)]
    motion_no_confidence: RealNumber,
    #[n(1)]
    update_committee: RealNumber,
    #[n(2)]
    update_committee_no_confidence: RealNumber,
    #[n(3)]
    hard_fork_initiation: RealNumber,
    #[n(4)]
    security_protocol_parameter_voting: RealNumber,
}
