use crate::{
    byron::{Attributes, protocol},
    crypto,
};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Data {
    pub version: protocol::Version,
    pub software_version: protocol::version::Software,
    pub attributes: Attributes,
    pub extra_proof: crypto::Blake2b256Digest,
}
