use minicbor::{CborLen, Decode, Encode};

pub mod plutus;
pub mod native;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(flat)]
pub enum Script {
    #[n(0)]
    Native(#[n(0)] native::Script),
    #[n(1)]
    PlutusV1(#[n(0)] plutus::Script),
    #[n(2)]
    PlutusV2(#[n(0)] plutus::Script),
    #[n(3)]
    PlutusV3(#[n(0)] plutus::Script),
}
