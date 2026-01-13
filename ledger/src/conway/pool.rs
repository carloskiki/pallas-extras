use std::net::{Ipv4Addr, Ipv6Addr};

use minicbor::{CborLen, Decode, Encode};

use crate::{crypto::Blake2b256Digest, protocol::RealNumber};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Metadata {
    #[cbor(n(0), with = "cbor_util::url")]
    pub url: Box<str>,
    #[cbor(n(1), with = "minicbor::bytes")]
    pub metadata_hash: Blake2b256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(flat)]
pub enum Relay {
    #[n(0)]
    HostAddress {
        #[n(0)]
        port: Option<u16>,
        #[n(1)]
        ipv4: Option<Ipv4Addr>,
        #[n(2)]
        ipv6: Option<Ipv6Addr>,
    },
    #[n(1)]
    HostName {
        #[n(0)]
        port: Option<u16>,
        #[cbor(n(1), with = "cbor_util::url")]
        dns_name: Box<str>,
    },
    #[n(2)]
    MultiHostName {
        #[cbor(n(0), with = "cbor_util::url")]
        dns_name: Box<str>,
    },
}

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
