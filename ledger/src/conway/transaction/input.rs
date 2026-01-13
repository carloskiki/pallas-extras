use tinycbor_derive::{Encode, Decode, CborLen};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Input<'a> {
    pub id: &'a super::Id,
    pub index: u16,
}
