use tinycbor_derive::{CborLen, Decode, Encode};

pub mod body;
pub use body::Body;

pub mod data;
pub use data::Data;

pub mod witness;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Transaction<'a> {
    pub body: body::Body<'a>,
    pub witnesses: witness::Set<'a>,
    pub data: Option<data::Data<'a>>,
}
