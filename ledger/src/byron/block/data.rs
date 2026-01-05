use crate::{
    byron::{Attributes, protocol},
    crypto,
};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Data<'a> {
    pub version: protocol::Version,
    pub software_version: protocol::version::Software<'a>,
    pub attributes: Attributes<'a>,
    pub extra_proof: &'a crypto::Blake2b256Digest,
}
