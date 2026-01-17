use crate::{
    shelley::{
        Certificate, Update,
        address::Account,
        transaction::{Coin, Input, Output},
    },
    slot,
};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Body<'a> {
    #[n(0)]
    pub inputs: Vec<Input<'a>>,
    #[n(1)]
    pub outputs: Vec<Output<'a>>,
    #[n(2)]
    pub fee: Coin,
    #[n(3)]
    pub ttl: slot::Number,
    #[cbor(n(4), optional)]
    pub certificates: Vec<Certificate<'a>>,
    #[cbor(n(5), optional)]
    pub withdrawals: Vec<(Account<'a>, Coin)>,
    #[cbor(n(6), optional, decode_with = "Update<'a>")]
    pub update: Option<Update<'a>>,
    #[cbor(n(7), optional, decode_with = "&'a crate::crypto::Blake2b256Digest")]
    pub auxiliary_data_hash: Option<&'a crate::crypto::Blake2b256Digest>,
}
