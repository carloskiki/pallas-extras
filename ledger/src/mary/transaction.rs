use tinycbor_derive::{CborLen, Decode, Encode};

pub mod body;
pub use body::Body;

pub mod output;
pub use output::Output;

pub mod value;
pub use value::Value;

use crate::allegra::transaction::{data, witness};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Transaction<'a> {
    pub body: body::Body<'a>,
    pub witness: witness::Set<'a>,
    pub data: Option<data::Data<'a>>,
}
