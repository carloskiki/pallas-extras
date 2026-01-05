use mitsein::vec1::Vec1;
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod fee_policy;
pub use fee_policy::FeePolicy;

pub mod input;
pub use input::Input;

pub mod output;
pub use output::Output;

pub mod payload;
pub use payload::Payload;

pub mod proof;
pub use proof::Proof;

pub mod witness;
pub use witness::Witness;

pub type Id = crate::crypto::Blake2b256Digest;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Transaction<'a> {
    #[cbor(with = "cbor_util::NonEmpty<Vec<Input<'a>>>")]
    pub inputs: Vec1<Input<'a>>,
    #[cbor(with = "cbor_util::NonEmpty<Vec<Output<'a>>>")]
    pub outputs: Vec1<Output<'a>>,
    pub attributes: super::Attributes<'a>,
}
