use crate::{
    byron::block::{self, Difficulty},
    byron::crypto::ExtendedVerifyingKey,
};
use tinycbor_derive::{CborLen, Decode, Encode};

mod slot;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Data<'a> {
    pub slot: slot::Id,
    #[cbor(with = "cbor_util::ExtendedVerifyingKey<'a>")]
    pub genesis_key: &'a ExtendedVerifyingKey,
    pub difficulty: [Difficulty; 1],
    pub signature: block::Signature<'a>,
}

