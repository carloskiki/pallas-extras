use tinycbor_derive::{CborLen, Decode, Encode};

use crate::{
    conway::governance::{Action, Anchor},
    shelley::{address::Account},
    Coin
};

// TODO: check if this should be owned.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Procedure<'a> {
    pub deposit: Coin,
    pub account: Account,
    pub action: Action<'a>,
    pub anchor: Anchor<'a>,
}
