use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct VersionData {
    pub network_magic: crate::NetworkMagic,
    pub query: bool,
}
