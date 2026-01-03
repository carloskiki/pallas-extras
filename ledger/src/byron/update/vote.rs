use crate::crypto;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Vote {
    #[cbor(with = "cbor_util::ExtendedVerifyingKey")]
    pub voter: crypto::ExtendedVerifyingKey,
    pub proposal_id: super::proposal::Id,
    pub vote: bool,
    #[cbor(with = "cbor_util::Signature<crypto::Signature>")]
    pub signature: crypto::Signature,
}
