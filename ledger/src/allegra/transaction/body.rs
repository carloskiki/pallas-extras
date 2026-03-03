use crate::{
    Unique,
    allegra::Update,
    shelley::{
        Certificate,
        address::Account,
        transaction::{Coin, Input, Output},
    },
    slot, unique,
};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Body<'a> {
    #[cbor(n(0), decode_with = "unique::codec::Set<Input<'a>>")]
    pub inputs: Unique<Vec<Input<'a>>, false>,
    #[n(1)]
    pub outputs: Vec<Output<'a>>,
    #[n(2)]
    pub fee: Coin,
    #[cbor(n(3), optional, decode_with = "slot::Number")]
    pub ttl: Option<slot::Number>,
    #[cbor(n(4), optional)]
    pub certificates: Vec<Certificate<'a>>,
    #[cbor(n(5), optional)]
    pub withdrawals: Unique<Vec<(Account<'a>, Coin)>, false>,
    #[cbor(n(6), optional, decode_with = "Update<'a>")]
    pub update: Option<Update<'a>>,
    #[cbor(n(7), optional, decode_with = "&'a crate::crypto::Blake2b256Digest")]
    pub auxiliary_data_hash: Option<&'a crate::crypto::Blake2b256Digest>,
    #[cbor(n(8), optional, decode_with = "slot::Number")]
    pub validity_start: Option<slot::Number>,
}
