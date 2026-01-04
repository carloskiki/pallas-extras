use tinycbor_derive::{CborLen, Decode, Encode};

pub mod data;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Witness {
    #[n(0)]
    VerifyingKey(#[cbor(with = "tinycbor::Encoded<data::VerifyingKey>")] data::VerifyingKey),
    #[n(2)]
    Redeemer(#[cbor(with = "tinycbor::Encoded<data::Redeemer>")] data::Redeemer),
}
