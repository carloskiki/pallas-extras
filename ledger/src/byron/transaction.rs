use hashbrown::HashMap;
use mitsein::vec1::Vec1;
use tinycbor_derive::{CborLen, Decode, Encode};
use yoke::Yoke;

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
pub struct Transaction<'a> {
    #[cbor(with = "crate::transaction::input::codec::ByronInputs<'a>")]
    pub inputs: Vec1<Input<'a>>,
    #[cbor(with = "cbor_util::NonEmpty<Vec<Output<'a>>>")]
    pub outputs: Vec1<Output<'a>>,
    pub attributes: super::Attributes<'a>,
}

pub type State = HashMap<transaction::input::Owned, Yoke<Output<'static>, Box<[u8]>>>;

pub fn transition(state: &mut State, transaction: &Payload<'_>) -> Result
