use crate::{allegra, alonzo::script::PlutusV1};
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod cost;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Decode, Encode, CborLen)]
pub enum Script<'a> {
    #[n(0)]
    Native(allegra::Script<'a>),
    #[n(1)]
    PlutusV1(&'a PlutusV1),
    #[n(2)]
    PlutusV2(&'a PlutusV2),
}

pub type PlutusV2 = [u8];
