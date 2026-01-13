use crate::conway::{
    script::{self, Script},
    transaction::datum::Datum,
};
use displaydoc::Display;
use thiserror::Error;
use tinycbor::{
    Decode, Encoded,
    collections::{self, fixed},
    tag,
};
use tinycbor_derive::{CborLen, Decode, Encode};

use super::Value;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, CborLen)]
#[cbor(map)]
pub struct Output<'a> {
    #[n(0)]
    pub address: crate::Address<'a>,
    #[n(1)]
    pub value: Value<'a>,
    #[cbor(n(2), optional)]
    pub datum: Option<Datum<'a>>,
    #[cbor(n(3), optional, with = "Encoded<Option<Script<'a>>>")]
    pub script_ref: Option<Script<'a>>,
}

#[derive(Debug, Error, Display)]
pub enum Error {
    /// while decoding `address`
    Address(#[from] <crate::Address<'static> as Decode<'static>>::Error),
    /// while decoding `value`
    Value(#[from] <Value<'static> as Decode<'static>>::Error),
    /// while decoding `datum`
    Datum(#[from] <Datum<'static> as Decode<'static>>::Error),
    /// while decoding `script_ref`
    ScriptRef(<Encoded<Script<'static>> as Decode<'static>>::Error),
}

// type Thing = <Encoded<Script<'static>> as Decode<'static>>::Error;

// type ThingE =
//     tag::Error<collections::Error<collections::Error<fixed::Error<tag::Error<script::Error>>>>>;
// 
// impl ::core::convert::From<ThingE> for Error {
//     fn from(source: <Encoded<Script<'static>> as Decode<'static>>::Error) -> Self {
//         Error::ScriptRef(source)
//     }
// }

// impl<'a, 'b: 'a> Decode<'b> for Output<'a> {
//     type Error = collections::Error<fixed::Error<>>;
//
//     fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

// struct EncodedScript<'a>(Encoded<Script<'a>>);
//
// impl<'a> From<EncodedScript<'a>> for Option<Script<'a>> {
//     fn from(encoded: EncodedScript<'a>) -> Self {
//         Some(encoded.0.0)
//     }
// }
//
// impl<'a> Decode<'a> for EncodedScript<'a> {
//     type Error = <Encoded<Script<'a>> as Decode<'a>>::Error;
//
//     fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
//         Ok(EncodedScript(Encoded::decode(d)?))
//     }
// }
