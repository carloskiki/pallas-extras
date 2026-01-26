use tinycbor_derive::{CborLen, Decode, Encode};

use crate::{
    conway::{governance::{Action, Anchor}},
    shelley::{address::Account, transaction::Coin},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Procedure<'a> {
    pub deposit: Coin,
    pub account: Account<'a>,
    pub action: Action<'a>,
    pub anchor: Anchor<'a>,
}
