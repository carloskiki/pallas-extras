use crate::{
    byron::{Attributes, protocol},
    crypto,
};
use tinycbor_derive::{CborLen, Decode, Encode};

pub type Id = crypto::Blake2b256Digest;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Proposal {
    pub protocol_version: protocol::Version,
    pub modifications: protocol::Parameters,
    pub software_version: protocol::version::Software,
    pub data: Vec<(String, super::Data)>,
    pub attributes: Attributes,
    #[cbor(with = "cbor_util::ExtendedVerifyingKey")]
    pub issuer: crypto::ExtendedVerifyingKey,
    #[cbor(with = "cbor_util::Signature<crypto::Signature>")]
    pub signature: crypto::Signature,
}
