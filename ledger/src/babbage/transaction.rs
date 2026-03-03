use tinycbor_derive::{CborLen, Decode, Encode};

pub mod body;
pub use body::Body;

pub mod data;
pub use data::Data;

pub mod datum;
pub use datum::Datum;

pub mod output;
pub use output::Output;

pub mod witness;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Transaction<'a> {
    pub body: Body<'a>,
    pub witnesses: witness::Set<'a>,
    pub valid: bool,
    pub data: Option<Data<'a>>,
}

// To allow `duplicate!` of `body` to conway era.
type SetCodec<T> = crate::unique::codec::Set<T>;
// To allow `duplicate!` of `ouptut`to conway era.
type Value<'a> = crate::mary::transaction::Value<'a>;
