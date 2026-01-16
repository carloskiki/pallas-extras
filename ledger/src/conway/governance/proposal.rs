use tinycbor_derive::{CborLen, Decode, Encode};

use crate::{
    conway::{governance::{Action, Anchor}, transaction::Coin},
    shelley::address::Account,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Proposal<'a> {
    pub deposit: Coin,
    pub account: Account<'a>,
    pub action: Action,
    pub anchor: Anchor<'a>,
}
