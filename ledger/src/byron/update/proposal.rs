use crate::{
    byron::{Attributes, protocol},
    crypto,
};
use tinycbor_derive::{CborLen, Decode, Encode};

pub type Id = crypto::Blake2b256Digest;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Proposal<'a> {
    pub protocol_version: protocol::Version,
    pub modifications: protocol::Parameters,
    pub software_version: protocol::version::Software<'a>,
    pub data: Vec<(&'a str, super::Data<'a>)>,
    pub attributes: Attributes<'a>,
    #[cbor(with = "cbor_util::ExtendedVerifyingKey<'a>")]
    pub issuer: &'a crypto::ExtendedVerifyingKey,
    #[cbor(with = "cbor_util::Bytes<'a, crypto::Signature>")]
    pub signature: &'a crypto::Signature,
}
