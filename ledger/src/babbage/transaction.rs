use tinycbor_derive::{CborLen, Decode, Encode};

pub mod body;
pub use body::Body;

pub mod data;
pub use data::Data;

pub mod datum;
pub use datum::Datum;

pub mod output;
pub use output::Output;
// So that we can duplicate the `Output` impl from here to the `conway` era.
type Value<'a> = crate::mary::transaction::Value<'a>;

pub mod witness;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Transaction<'a> {
    pub body: Body<'a>,
    pub witnesses: witness::Set<'a>,
    pub valid: bool,
    pub data: Option<Data<'a>>,
}
