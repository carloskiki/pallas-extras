use crate::{
    allegra,
    alonzo::{
        script::{Data, PlutusV1},
        transaction::Redeemer,
    },
    babbage::script::PlutusV2,
    shelley::transaction::witness::{Bootstrap, VerifyingKey},
};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Set<'a> {
    #[cbor(n(0), optional)]
    pub verifying_keys: Box<[VerifyingKey<'a>]>,
    #[cbor(n(1), optional)]
    pub native_scripts: Box<[allegra::Script<'a>]>,
    #[cbor(n(2), optional)]
    pub bootstraps: Box<[Bootstrap<'a>]>,
    #[cbor(n(3), optional)]
    pub plutus_v1: Box<[&'a PlutusV1]>,
    #[cbor(n(4), optional)]
    pub plutus_data: Box<[Data]>,
    #[cbor(n(5), optional)]
    pub redeemers: Box<[Redeemer]>,
    #[cbor(n(6), optional)]
    pub plutus_v2: Box<[&'a PlutusV2]>,
}
