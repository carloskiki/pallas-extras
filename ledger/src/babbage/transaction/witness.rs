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
    pub verifying_keys: Vec<VerifyingKey<'a>>,
    #[cbor(n(1), optional)]
    pub native_scripts: Vec<allegra::Script<'a>>,
    #[cbor(n(2), optional)]
    pub bootstraps: Vec<Bootstrap<'a>>,
    #[cbor(n(3), optional)]
    pub plutus_v1: Vec<&'a PlutusV1>,
    #[cbor(n(4), optional)]
    pub plutus_data: Vec<Data>,
    #[cbor(n(5), optional)]
    pub redeemers: Vec<Redeemer>,
    #[cbor(n(6), optional)]
    pub plutus_v2: Vec<&'a PlutusV2>,
}
