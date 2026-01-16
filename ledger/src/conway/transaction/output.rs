use crate::conway::{
    script::{self, Script},
    transaction::datum::Datum,
};
use displaydoc::Display;
use thiserror::Error;
use tinycbor::{
    Decode, Encoded, Type,
    collections::{self, fixed},
    primitive, tag,
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
    ScriptRef(#[from] ScriptRefError),
    /// while decoding a map key
    Key(#[from] primitive::Error),
}

// This is equivalent to:
// ```
// type ScriptRefError = <Encoded<Script<'static>> as Decode<'static>>::Error;
// ```
// but the compiler does not like this.
//
// My guess is that the compiler can't see
// `<Encoded<Script<'static>> as Decode<'static>>::Error != Error`
// but I don't know why.
type ScriptRefError =
    tag::Error<collections::Error<collections::Error<fixed::Error<tag::Error<script::Error>>>>>;

impl<'a, 'b: 'a> Decode<'b> for Output<'a> {
    type Error = collections::Error<fixed::Error<Error>>;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        match d.datatype()? {
            Type::Array | Type::ArrayIndef => {
                let mut v = d.array_visitor()?;
                let address: crate::Address = v
                    .visit()
                    .ok_or(collections::Error::Element(fixed::Error::Missing))?
                    .map_err(|e| {
                        collections::Error::Element(fixed::Error::Inner(Error::Address(e)))
                    })?;
                let value: Value = v
                    .visit()
                    .ok_or(collections::Error::Element(fixed::Error::Missing))?
                    .map_err(|e| {
                        collections::Error::Element(fixed::Error::Inner(Error::Value(e)))
                    })?;
                let datum: Option<Datum> = v.visit().transpose().map_err(|e| {
                    collections::Error::Element(fixed::Error::Inner(Error::Datum(e)))
                })?;
                if v.remaining() != Some(0) {
                    return Err(collections::Error::Element(fixed::Error::Surplus));
                }
                Ok(Output {
                    address,
                    value,
                    datum,
                    script_ref: None,
                })
            }
            _ => {
                let Codec { address, value, datum, script_ref }: Codec<'a> =
                    Decode::decode(d).map_err(|e: collections::Error<_>| e.map(|e: fixed::Error<_>| e.map(|e| match e {
                        collections::map::Error::Key(k) => Error::Key(k),
                        collections::map::Error::Value(e) => match e {
                            CodecError::Address(a) => Error::Address(a),
                            CodecError::Value(v) => Error::Value(v),
                            CodecError::Datum(d) => Error::Datum(d),
                            CodecError::ScriptRef(s) => Error::ScriptRef(s),
                        },
                    })))?;
                Ok(Output {
                    address,
                    value,
                    datum,
                    script_ref,
                })
                
            }
        }
    }
}

#[derive(Decode)]
#[cbor(map, error = "CodecError")]
struct Codec<'a> {
    #[n(0)]
    pub address: crate::Address<'a>,
    #[n(1)]
    pub value: Value<'a>,
    #[cbor(n(2), optional)]
    pub datum: Option<Datum<'a>>,
    #[cbor(n(3), optional, decode_with = "EncodedScript<'_>")]
    pub script_ref: Option<Script<'a>>,
}

struct EncodedScript<'a>(Encoded<Script<'a>>);

impl<'a> From<EncodedScript<'a>> for Option<Script<'a>> {
    fn from(encoded: EncodedScript<'a>) -> Self {
        Some(encoded.0.0)
    }
}

impl<'a> Decode<'a> for EncodedScript<'a> {
    type Error = <Encoded<Script<'a>> as Decode<'a>>::Error;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        Ok(EncodedScript(Encoded::decode(d)?))
    }
}
