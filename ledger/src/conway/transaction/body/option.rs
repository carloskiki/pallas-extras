use crate::{
    conway::{
        asset,
        Asset,
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
    slot,
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
    Certificates(Vec<Certificate<'a>>),
    #[n(5)]
    Withdrawals(Vec<(Account<'a>, Coin)>),
    #[n(7)]
    AuxiliaryDataHash(&'a Blake2b256Digest),
    #[n(8)]
    ValidityStart(slot::Number),
    #[n(9)]
    Mint(#[cbor(with = "asset::Codec<'a, NonZero<i64>>")] Asset<'a, NonZero<i64>>),
    #[n(11)]
    ScriptDataHash(&'a Blake2b256Digest),
    #[n(13)]
    Collateral(Vec<Input<'a>>),
    #[n(14)]
    RequiredSigners(Vec<&'a Blake2b224Digest>),
    #[n(15)]
    Network(Network),
    #[n(16)]
    CollateralReturn(Output<'a>),
    #[n(17)]
    CollateralAmount(Coin),
    #[n(18)]
    ReferenceInputs(Vec<Input<'a>>),
    #[n(19)]
    VotingProcedures(#[cbor(with = "voting::Codec<'a>")] voting::Procedures<'a>),
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
