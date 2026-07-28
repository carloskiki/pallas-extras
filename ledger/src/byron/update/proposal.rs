use crate::{
    byron::{Attributes, crypto::ExtendedVerifyingKey, protocol},
    crypto::{Blake2b256Digest, Signature},
};
use tinycbor_derive::{CborLen, Decode, Encode};

pub type Id = Blake2b256Digest;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Proposal<'a> {
    pub protocol_version: protocol::Version,
    pub modifications: protocol::parameter::Update,
    pub software_version: protocol::version::Software<'a>,
    pub data: Box<[(&'a str, super::Data<'a>)]>,
    pub attributes: Attributes,
    #[cbor(with = "cbor_util::ExtendedVerifyingKey<'a>")]
    pub issuer: &'a ExtendedVerifyingKey,
    #[cbor(with = "cbor_util::Signature<'a>")]
    pub signature: &'a Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Registration {
    pub version: protocol::Version,
    pub parameters: protocol::Parameters,
}
