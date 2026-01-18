use tinycbor_derive::{Decode, Encode, CborLen};
use crate::{crypto, shelley::{block, certificate, protocol}, slot};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Body<'a> {
    pub number: block::Number,
    pub slot: slot::Number,
    pub previous: Option<&'a block::Id>,
    #[cbor(with = "cbor_util::VerifyingKey<'a>")]
    pub issuer: &'a crypto::VerifyingKey,
    #[cbor(with = "cbor_util::VerifyingKey<'a>")]
    pub vrf: &'a crypto::VerifyingKey,
    pub nonce_vrf: certificate::Vrf<'a>,
    pub leader_vrf: certificate::Vrf<'a>,
    pub size: block::Size,
    pub body_hash: &'a crypto::Blake2b256Digest,
    /// KES "hot" verifying key
    #[cbor(with = "cbor_util::Bytes<'a, crypto::kes::VerifyingKey>")]
    pub signer: &'a crypto::kes::VerifyingKey,
    /// KES sequence number
    pub sequence_number: u32,
    /// KES period
    pub period: u32,
    /// signature for certificate
    #[cbor(with = "cbor_util::Signature<'a>")]
    pub signature: &'a crypto::Signature,
    pub fork: protocol::version::Fork,
    #[cbor(with = "tinycbor::num::U8")]
    pub minor: u8,
}
