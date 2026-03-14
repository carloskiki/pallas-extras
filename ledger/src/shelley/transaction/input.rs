use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Input<'a> {
    pub id: &'a crate::byron::transaction::Id,
    pub index: super::Index,
}
