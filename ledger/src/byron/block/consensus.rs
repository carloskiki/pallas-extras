use tinycbor_derive::{Encode, Decode, CborLen};
use crate::{slot, crypto, byron::block::Difficulty};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Data {
    pub slot: slot::Number,
    #[cbor(with = "cbor_util::ExtendedVerifyingKey")]
    pub genesis_key: crypto::ExtendedVerifyingKey,
    pub difficulty: [Difficulty; 1],
    #[cbor(with = "cbor_util::Signature<crypto::Signature>")]
    pub signature: crypto::Signature,
}
