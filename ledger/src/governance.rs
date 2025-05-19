pub mod action;
pub mod delegate_representative;
pub mod voting;

use std::fmt::Debug;

use minicbor::{CborLen, Decode, Encode};

pub use action::Action;
pub use delegate_representative::DelegateRepresentative;

use crate::{
    address::shelley::StakeAddress,
    crypto::{Blake2b224Digest, Blake2b256Digest},
    transaction::Coin,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Anchor {
    #[cbor(n(0), with = "cbor_util::url")]
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

pub(crate) fn dbg_d<'a, C,T: Decode<'a, C> + Debug>(
    d: &mut minicbor::Decoder<'a>,
    ctx: &mut C,
) -> Result<T, minicbor::decode::Error> {
    dbg!("pre dbg");
    let value = d.decode_with(ctx)?;
    dbg!(&value);
    Ok(value)
}
