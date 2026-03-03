use crate::{
    Unique, allegra,
    alonzo::script::{Data, PlutusV1},
    babbage::script::PlutusV2,
    conway::{script::PlutusV3, transaction::Redeemers},
    shelley::transaction::witness::{Bootstrap, VerifyingKey},
    unique,
};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Set<'a> {
    #[cbor(
        n(0),
        optional,
        decode_with = "unique::codec::NonEmpty<VerifyingKey<'a>>"
    )]
    pub verifying_keys: Unique<Vec<VerifyingKey<'a>>, false>,
    #[cbor(
        n(1),
        optional,
        decode_with = "unique::codec::NonEmpty<allegra::Script<'a>>"
    )]
    pub native_scripts: Unique<Vec<allegra::Script<'a>>, false>,
    #[cbor(n(2), optional, decode_with = "unique::codec::NonEmpty<Bootstrap<'a>>")]
    pub bootstraps: Unique<Vec<Bootstrap<'a>>, false>,
    #[cbor(n(3), optional, decode_with = "unique::codec::NonEmpty<&'a PlutusV1>")]
    pub plutus_v1: Unique<Vec<&'a PlutusV1>, false>,
    #[cbor(n(4), optional, decode_with = "unique::codec::NonEmpty<Data>")]
    pub plutus_data: Unique<Vec<Data>, false>,
    #[cbor(n(5), optional)]
    pub redeemers: Redeemers,
    #[cbor(n(6), optional, decode_with = "unique::codec::NonEmpty<&'a PlutusV2>")]
    pub plutus_v2: Unique<Vec<&'a PlutusV2>, false>,
    #[cbor(n(7), optional, decode_with = "unique::codec::NonEmpty<&'a PlutusV3>")]
    pub plutus_v3: Unique<Vec<&'a PlutusV3>, false>,
}
