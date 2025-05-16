pub mod action;
pub mod voting;
pub mod delegate_representative;

use minicbor::{CborLen, Decode, Encode};

pub use action::Action;
pub use delegate_representative::DelegateRepresentative;

use crate::{
    address::shelley::StakeAddress,
    crypto::{Blake2b224Digest, Blake2b256Digest}, transaction::Coin,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Anchor {
    #[n(0)]
    url: Box<str>,
    #[cbor(n(1), with = "minicbor::bytes")]
    data_hash: Blake2b256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Constitution {
    #[n(0)]
    pub anchor: Anchor,
    #[cbor(n(1), with = "minicbor::bytes")]
    pub script_hash: Option<Blake2b224Digest>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Proposal {
    #[n(0)]
    pub deposit: Coin,
    #[n(1)]
    pub account: StakeAddress,
    #[n(2)]
    pub action: Action,
    #[n(3)]
    pub anchor: Anchor,
}
