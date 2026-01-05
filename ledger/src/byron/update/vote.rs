use crate::crypto;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Vote<'a> {
    #[cbor(with = "cbor_util::ExtendedVerifyingKey<'a>")]
    pub voter: &'a crypto::ExtendedVerifyingKey,
    pub proposal_id: &'a super::proposal::Id,
    pub vote: bool,
    #[cbor(with = "cbor_util::Bytes<'a, crypto::Signature>")]
    pub signature: &'a crypto::Signature,
}
