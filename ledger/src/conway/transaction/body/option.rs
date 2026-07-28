use crate::{
    Coin, Unique,
    conway::{
        Asset, Certificate, asset,
        governance::{
            proposal,
            voting::{self},
        },
        transaction::Output,
    },
    crypto::{Blake2b224Digest, Blake2b256Digest},
    shelley::{Network, address::Account},
    slot,
    transaction::Reference,
    unique,
};
use mitsein::boxed1::BoxedSlice1;
use sparse_struct::SparseStruct;
use std::num::NonZero;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen, SparseStruct,
)]
#[sparse_name = "Options"]
#[struct_derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cbor(naked)]
pub enum Option<'a> {
    #[n(3)]
    TimeToLive(slot::Number),
    #[n(4)]
    Certificates(
        #[cbor(with = "unique::codec::NonEmpty<Certificate<'a>>")]
        Unique<BoxedSlice1<Certificate<'a>>, false>,
    ),
    #[n(5)]
    Withdrawals(
        #[cbor(
            encode_with = "unique::codec::NonEmpty<(Account, Coin)>",
            len_with = "unique::codec::NonEmpty<(Account, Coin)>"
        )]
        Unique<BoxedSlice1<(Account, Coin)>, false>,
    ),
    #[n(7)]
    AuxiliaryDataHash(&'a Blake2b256Digest),
    #[n(8)]
    ValidityStart(slot::Number),
    #[n(9)]
    Mint(#[cbor(with = "asset::Codec<'a, NonZero<i64>>")] Asset<'a, NonZero<i64>>),
    #[n(11)]
    ScriptDataHash(&'a Blake2b256Digest),
    #[n(13)]
    Collateral(
        #[cbor(with = "unique::codec::NonEmpty<Reference>")] Unique<BoxedSlice1<Reference>, false>,
    ),
    #[n(14)]
    RequiredSigners(
        #[cbor(with = "unique::codec::NonEmpty<&'a Blake2b224Digest>")]
        Unique<BoxedSlice1<&'a Blake2b224Digest>, false>,
    ),
    #[n(15)]
    Network(Network),
    #[n(16)]
    CollateralReturn(Output<'a>),
    #[n(17)]
    CollateralAmount(Coin),
    #[n(18)]
    ReferenceInputs(
        #[cbor(with = "unique::codec::NonEmpty<Reference>")] Unique<BoxedSlice1<Reference>, false>,
    ),
    #[n(19)]
    VotingProcedures(
        #[cbor(encode_with = "voting::Codec<'a>", len_with = "voting::Codec<'a>")]
        voting::Procedures<'a>,
    ),
    #[n(20)]
    ProposalProcedures(
        #[cbor(with = "unique::codec::NonEmpty<proposal::Procedure<'a>>")]
        Unique<BoxedSlice1<proposal::Procedure<'a>>, false>,
    ),
    #[n(21)]
    CurrentTreasury(Coin),
    #[n(22)]
    Donation(NonZero<Coin>),
}
