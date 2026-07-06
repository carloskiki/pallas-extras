use mitsein::vec1::Vec1;
use tinycbor_derive::{CborLen, Decode, Encode};

pub use crate::transaction::{self, Input};

pub mod output;
pub use output::Output;

pub mod payload;
pub use payload::Payload;

pub mod proof;
pub use proof::Proof;

pub mod witness;
pub use witness::Witness;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Transaction {
    #[cbor(with = "crate::transaction::input::codec::ByronInputs")]
    pub inputs: Vec1<Input>,
    #[cbor(with = "cbor_util::NonEmpty<Vec<Output>>")]
    pub outputs: Vec1<Output>,
    pub attributes: super::Attributes,
}
