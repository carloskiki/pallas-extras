use tinycbor_derive::{CborLen, Decode, Encode};

pub mod vote;
pub use vote::Vote;

pub mod proposal;
pub use proposal::Proposal;

pub mod data;
pub use data::Data;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Update<'a> {
    #[cbor(with = "cbor_util::option::Array<Proposal<'a>, false>")]
    pub proposal: Option<Proposal<'a>>,
    pub votes: Vec<Vote<'a>>,
}
