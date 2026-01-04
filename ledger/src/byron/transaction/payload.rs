use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Payload<'a> {
    transaction: super::Transaction<'a>,
    witnesses: Vec<super::Witness>,
}
