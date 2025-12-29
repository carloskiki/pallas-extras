use tinycbor_derive::{CborLen, Decode, Encode};

pub mod input;
pub use input::Input;

pub mod output;
pub use output::Output;

pub mod proof;
pub use proof::Proof;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Transaction {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub attributes: super::Attributes,
}
