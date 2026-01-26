use tinycbor_derive::{CborLen, Decode, Encode};

pub mod body;
pub use body::Body;

pub mod data;
pub use data::Data;

pub mod output;
pub use output::Output;

pub mod redeemer;
pub use redeemer::Redeemers;

pub mod value;
pub use value::Value;

pub mod witness;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Transaction<'a> {
    pub body: Body<'a>,
    pub witnesses: witness::Set<'a>,
    pub valid: bool,
    pub data: Option<Data<'a>>,
}
