use crate::{
    Address,
    babbage::{self, transaction},
};
use super::Value;
use displaydoc::Display;
use thiserror::Error;
use tinycbor::Decode;
use tinycbor_derive::{CborLen, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, CborLen)]
#[cbor(map)]
pub struct Output<'a> {
    #[n(0)]
    pub address: Address<'a>,
    #[n(1)]
    pub value: Value<'a>,
    #[cbor(n(2), optional)]
    pub datum: Option<transaction::Datum<'a>>,
    #[cbor(n(3), optional)]
    pub script: Option<babbage::Script<'a>>,
}

#[derive(Debug, Error, Display)]
pub enum Error {
    /// while decoding alonzo style `Output`
    Alonzo(#[from] <alonzo_style::Output<'static> as Decode<'static>>::Error),
    /// while decoding babbage style `Output`
    Babbage(#[from] <codec::Codec<'static> as Decode<'static>>::Error),
}

impl<'a, 'b: 'a> Decode<'b> for Output<'a> {
    type Error = Error;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        match d.datatype() {
            Ok(tinycbor::Type::Array | tinycbor::Type::ArrayIndef) => {
                let alonzo_style::Output {
                    address,
                    value,
                    datum_hash,
                } = alonzo_style::Output::decode(d)?;
                Ok(Output {
                    address,
                    value,
                    datum: datum_hash.map(transaction::Datum::Hash),
                    script: None,
                })
            }
            _ => {
                let codec::Codec {
                    address,
                    value,
                    datum,
                    script,
                } = codec::Codec::decode(d)?;
                Ok(Output {
                    address,
                    value,
                    datum,
                    script,
                })
            }
        }
    }
}

mod codec {
    use crate::{
        Address,
        babbage::{self, transaction::Datum},
    };
    use super::Value;
    use tinycbor::Encoded;
    use tinycbor_derive::{CborLen, Decode, Encode};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
    #[cbor(map)]
    pub struct Codec<'a> {
        #[n(0)]
        pub address: Address<'a>,
        #[n(1)]
        pub value: Value<'a>,
        #[cbor(n(2), optional, decode_with = "Datum<'a>")]
        pub datum: Option<Datum<'a>>,
        #[cbor(n(3), optional, decode_with = "EncodedScript<'a>")]
        pub script: Option<babbage::Script<'a>>,
    }

    #[derive(Decode)]
    #[cbor(naked)]
    pub struct EncodedScript<'a>(pub Encoded<babbage::Script<'a>>);

    impl<'a> From<EncodedScript<'a>> for Option<babbage::Script<'a>> {
        fn from(encoded: EncodedScript<'a>) -> Self {
            Some(encoded.0.0)
        }
    }
}

// We do not use `alonzo::transaction::Output` because it allows for oversized addresses (by
// truncating). starting with the `babbage` era, address decoding is strict.
mod alonzo_style {
    use super::super::Value;
    use std::convert::Infallible;
    use tinycbor::{
        Decode, Decoder, container::{self, bounded}
    };

    pub struct Output<'a> {
        pub address: crate::Address<'a>,
        pub value: Value<'a>,
        pub datum_hash: Option<&'a crate::crypto::Blake2b256Digest>,
    }

    #[derive(Debug, thiserror::Error, displaydoc::Display)]
    pub enum Error {
        /// in field `address`
        Address(#[from] <crate::Address<'static> as Decode<'static>>::Error),
        /// in field `value`
        Value(#[from] <Value<'static> as Decode<'static>>::Error),
        /// in field `datum_hash`
        DatumHash(#[from] container::Error<bounded::Error<Infallible>>),
    }

    impl<'a, 'b: 'a> Decode<'b> for Output<'a> {
        type Error = container::Error<bounded::Error<Error>>;

        fn decode(d: &mut Decoder<'b>) -> Result<Self, Self::Error> {
            fn wrap(e: impl Into<Error>) -> bounded::Error<Error> {
                bounded::Error::Content(e.into())
            }

            let mut visitor = d.array_visitor()?;
            let address: crate::Address = visitor
                .visit()
                .ok_or(bounded::Error::Missing)?
                .map_err(wrap)?;
            let value = visitor
                .visit()
                .ok_or(bounded::Error::Missing)?
                .map_err(wrap)?;
            let datum_hash = visitor.visit().map(|opt| opt.map_err(wrap)).transpose()?;
            if visitor.remaining() != Some(0) {
                return Err(bounded::Error::Surplus.into());
            }

            Ok(Self {
                address,
                value,
                datum_hash,
            })
        }
    }
}
