use crate::{
    Unique,
    conway::{
        Asset, asset,
        governance::{
            proposal,
            voting::{self},
        },
        transaction::Output,
    },
    crypto::{Blake2b224Digest, Blake2b256Digest},
    shelley::{
        Certificate, Network,
        address::Account,
        transaction::{Coin, Input},
    },
    slot, unique,
};
use mitsein::vec1::Vec1;
use sparse_struct::SparseStruct;
use std::num::NonZero;
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
    Certificates(
        #[cbor(with = "unique::codec::NonEmpty<Certificate<'a>>")]
        Unique<Vec1<Certificate<'a>>, false>,
    ),
    #[n(5)]
    Withdrawals(
        #[cbor(
            encode_with = "unique::codec::NonEmpty<(Account<'a>, Coin)>",
            len_with = "unique::codec::NonEmpty<(Account<'a>, Coin)>"
        )]
        Unique<Vec1<(Account<'a>, Coin)>, false>,
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
    Collateral(#[cbor(with = "unique::codec::NonEmpty<Input<'a>>")] Unique<Vec1<Input<'a>>, false>),
    #[n(14)]
    RequiredSigners(
        #[cbor(with = "unique::codec::NonEmpty<&'a Blake2b224Digest>")]
        Unique<Vec1<&'a Blake2b224Digest>, false>,
    ),
    #[n(15)]
    Network(Network),
    #[n(16)]
    CollateralReturn(Output<'a>),
    #[n(17)]
    CollateralAmount(Coin),
    #[n(18)]
    ReferenceInputs(
        #[cbor(with = "unique::codec::NonEmpty<Input<'a>>")] Unique<Vec1<Input<'a>>, false>,
    ),
    #[n(19)]
    VotingProcedures(
        #[cbor(encode_with = "voting::Codec<'a>", len_with = "voting::Codec<'a>")]
        voting::Procedures<'a>,
    ),
    #[n(20)]
    ProposalProcedures(
        #[cbor(with = "cbor_util::NonEmpty<Vec<proposal::Procedure<'a>>>")]
        Vec1<proposal::Procedure<'a>>,
    ),
    #[n(21)]
    CurrentTreasury(Coin),
    #[n(22)]
    Donation(NonZero<Coin>),
}
