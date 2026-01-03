use tinycbor_derive::{CborLen, Decode, Encode};

pub mod data;
pub use data::Data;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Witness {
    #[n(0)]
    VerifyingKey(#[cbor(with = "tinycbor::Encoded<Data>")] Data),
    #[n(2)]
    Redeemer(#[cbor(with = "tinycbor::Encoded<Data>")] Data),
}
