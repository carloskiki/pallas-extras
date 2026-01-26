use crate::{
    allegra,
    alonzo::script::{Data, PlutusV1},
    babbage::script::PlutusV2,
    conway::{script::PlutusV3, transaction::Redeemers},
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
    pub redeemers: Redeemers,
    #[cbor(n(6), optional)]
    pub plutus_v2: Vec<&'a PlutusV2>,
    #[cbor(n(7), optional)]
    pub plutus_v3: Vec<&'a PlutusV3>,
}
