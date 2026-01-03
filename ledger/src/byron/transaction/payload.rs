use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Payload {
    transaction: super::Transaction,
    witnesses: Vec<super::Witness>,
}
