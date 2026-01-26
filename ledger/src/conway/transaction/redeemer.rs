use crate::alonzo::script::{Data, execution};
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod index;
pub use index::Index;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Redeemer {
    pub data: Data,
    pub execution_units: execution::Units,
}

pub type Redeemers = Vec<(Index, Redeemer)>;

mod codec {
    use cbor_util::NonEmpty;
    use mitsein::{iter1::IntoIterator1, vec1::Vec1};
    use tinycbor::{
        Decode, Type,
        container::{self, map},
        num::nonzero,
    };

    pub struct Codec(pub super::Redeemers);

    impl From<Codec> for super::Redeemers {
        fn from(codec: Codec) -> Self {
            codec.0
        }
    }

    #[derive(Debug, thiserror::Error, displaydoc::Display)]
    pub enum Error {
        /// while decoding conway style `Redeemers`
        Conway(
            #[from]
            nonzero::Error<
                container::Error<
                    map::Error<
                        <super::Index as Decode<'static>>::Error,
                        <super::Redeemer as Decode<'static>>::Error,
                    >,
                >,
            >,
        ),
        /// while decoding alonzo style `Redeemers`
        Alonzo(
            #[from]
            nonzero::Error<container::Error<<super::legacy::Redeemer as Decode<'static>>::Error>>,
        ),
    }

    impl Decode<'_> for Codec {
        type Error = Error;

        fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
            match d.datatype() {
                Ok(Type::Array | Type::ArrayIndef) => {
                    let x: Vec1<_> = <NonEmpty<Vec<super::legacy::Redeemer>>>::decode(d)?
                        .0
                        .into_iter1()
                        .map(|r| {
                            let index = super::Index {
                                kind: r.kind,
                                index: r.index,
                            };
                            let redeemer = super::Redeemer {
                                data: r.data,
                                execution_units: r.execution_units,
                            };
                            (index, redeemer)
                        })
                        .collect1();
                    Ok(Codec(x.into()))
                }
                _ => Ok(NonEmpty::<Vec<(super::Index, super::Redeemer)>>::decode(d)
                    .map(|r| Codec(r.0.into()))?),
            }
        }
    }
}

mod legacy {
    use crate::alonzo::script::{Data, execution};

    #[derive(tinycbor_derive::Decode)]
    pub struct Redeemer {
        pub kind: super::index::Kind,
        pub index: u64,
        pub data: Data,
        pub execution_units: execution::Units,
    }
}
