use crate::{
    Coin, Unique,
    shelley::{Certificate, Update, address::Account, transaction::Output},
    slot,
    transaction::Input,
    unique,
};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Body<'a> {
    #[cbor(n(0), decode_with = "unique::codec::Set<Input>")]
    pub inputs: Unique<Vec<Input>, false>,
    #[n(1)]
    pub outputs: Vec<Output>,
    #[n(2)]
    pub fee: Coin,
    #[n(3)]
    pub ttl: slot::Number,
    #[cbor(n(4), optional)]
    pub certificates: Vec<Certificate<'a>>,
    #[cbor(n(5), optional)]
    pub withdrawals: Unique<Vec<(Account, Coin)>, false>,
    #[cbor(n(6), optional, decode_with = "Update<'a>")]
    pub update: Option<Update<'a>>,
    #[cbor(n(7), optional, decode_with = "&'a crate::crypto::Blake2b256Digest")]
    pub auxiliary_data_hash: Option<&'a crate::crypto::Blake2b256Digest>,
}
