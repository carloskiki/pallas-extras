use crate::shelley::url::Url;
use std::net::{Ipv4Addr, Ipv6Addr};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub enum Relay<'a> {
    #[n(0)]
    HostAddress {
        port: Option<u16>,
        #[cbor(with = "cbor_util::Ipv4Addr")]
        ipv4: Option<Ipv4Addr>,
        #[cbor(with = "cbor_util::Ipv6Addr")]
        ipv6: Option<Ipv6Addr>,
    },
    #[n(1)]
    HostName {
        port: Option<u16>,
        dns_name: &'a Url,
    },
    #[n(2)]
    MultiHostName { dns_name: &'a Url },
}
