use tinycbor_derive::{CborLen, Decode, Encode};

pub mod data;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Witness<'a> {
    #[n(0)]
    VerifyingKey(
        #[cbor(with = "tinycbor::Encoded<data::VerifyingKey<'a>>")] data::VerifyingKey<'a>,
    ),
    #[n(2)]
    Redeemer(#[cbor(with = "tinycbor::Encoded<data::Redeemer<'a>>")] data::Redeemer<'a>),
}
