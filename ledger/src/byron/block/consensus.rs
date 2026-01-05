use crate::{
    byron::block::{self, Difficulty},
    crypto, slot,
};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Data<'a> {
    pub slot: slot::Id,
    #[cbor(with = "cbor_util::ExtendedVerifyingKey<'a>")]
    pub genesis_key: &'a crypto::ExtendedVerifyingKey,
    pub difficulty: [Difficulty; 1],
    pub signature: block::Signature<'a>,
}
