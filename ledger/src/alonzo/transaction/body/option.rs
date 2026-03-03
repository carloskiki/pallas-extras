use crate::{
    Unique,
    alonzo::Update,
    crypto::{Blake2b224Digest, Blake2b256Digest},
    mary::{Asset, asset},
    shelley::{
        Certificate, Network,
        address::Account,
        transaction::{Coin, Input},
    },
    slot, unique,
};
use sparse_struct::SparseStruct;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen, SparseStruct,
)]
#[struct_name = "Options"]
#[struct_derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cbor(naked)]
pub enum Option<'a> {
    #[n(3)]
    TimeToLive(slot::Number),
    #[n(4)]
    Certificates(Vec<Certificate<'a>>),
    #[n(5)]
    Withdrawals(Unique<Vec<(Account<'a>, Coin)>, false>),
    #[n(6)]
    Update(Update<'a>),
    #[n(7)]
    AuxiliaryDataHash(&'a Blake2b256Digest),
    #[n(8)]
    ValidityStart(slot::Number),
    #[n(9)]
    Mint(#[cbor(with = "asset::Codec<'_, i64>")] Asset<'a, i64>),
    #[n(11)]
    ScriptDataHash(&'a Blake2b256Digest),
    #[n(13)]
    Collateral(
        #[cbor(decode_with = "unique::codec::Set<Input<'a>>")] Unique<Vec<Input<'a>>, false>,
    ),
    #[n(14)]
    RequiredSigners(
        #[cbor(decode_with = "unique::codec::Set<&'a Blake2b224Digest>")]
        Unique<Vec<&'a Blake2b224Digest>, false>,
    ),
    #[n(15)]
    Network(Network),
}
