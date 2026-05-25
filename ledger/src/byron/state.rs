use crate::{slot, byron::block};

pub struct State {
    pub slot: slot::Number,
    pub hash: block::Id,
    pub utxo: (), // TODO
    pub epoch: epoch::
}
