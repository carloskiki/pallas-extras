use crate::{
    mary::{
        Update,
        asset::{self, Asset},
        transaction::output::Output,
    },
    shelley::{
        Certificate,
        address::Account,
        transaction::{Coin, Input},
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
    #[cbor(n(3), optional, decode_with = "slot::Number")]
    pub ttl: Option<slot::Number>,
    #[cbor(n(4), optional)]
    pub certificates: Vec<Certificate<'a>>,
    #[cbor(n(5), optional)]
    pub withdrawals: Vec<(Account<'a>, Coin)>,
    #[cbor(n(6), optional, decode_with = "Update<'a>")]
    pub update: Option<Update<'a>>,
    #[cbor(n(7), optional, decode_with = "&'a crate::crypto::Blake2b256Digest")]
    pub auxiliary_data_hash: Option<&'a crate::crypto::Blake2b256Digest>,
    #[cbor(n(8), optional, decode_with = "slot::Number")]
    pub validity_start: Option<slot::Number>,
    #[cbor(n(9), optional, with = "asset::Codec<'_, i64>")]
    pub mint: Asset<'a, i64>,
}
