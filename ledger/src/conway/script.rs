use tinycbor_derive::{CborLen, Decode, Encode};

pub mod plutus;
pub mod native;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub enum Script<'a> {
    #[n(0)]
    Native(native::Script<'a>),
    #[n(1)]
    PlutusV1(plutus::Script<'a>),
    #[n(2)]
    PlutusV2(plutus::Script<'a>),
    #[n(3)]
    PlutusV3(plutus::Script<'a>),
}
