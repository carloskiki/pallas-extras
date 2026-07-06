use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Payload<'a> {
    pub transaction: super::Transaction,
    pub witnesses: Vec<super::Witness<'a>>,
}
