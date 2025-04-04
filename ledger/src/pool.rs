use std::net::{Ipv4Addr, Ipv6Addr};

use minicbor::{Decode, Encode};

use crate::crypto::Blake2b256Digest;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct Metadata {
    #[n(0)]
    pub url: String,
    #[cbor(n(1), with = "minicbor::bytes")]
    pub metadata_hash: Blake2b256Digest,
}

// TODO: what type to use for dns_name?
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
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
        #[n(1)]
        dns_name: String,
    },
    #[n(2)]
    MultiHostName {
        #[n(0)]
        dns_name: String,
    },
}
