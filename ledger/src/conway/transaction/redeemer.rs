use crate::{
    Unique,
    alonzo::script::{Data, execution},
};
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod index;
pub use index::Index;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Redeemer {
    pub data: Data,
    pub execution_units: execution::Units,
}

pub type Redeemers = Unique<Vec<(Index, Redeemer)>, false>;

mod codec {
    use mitsein::vec1::Vec1;
    use tinycbor::{Decode, Type};

    use crate::{Unique, unique};

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
            <Unique<Vec1<(super::Index, super::Redeemer)>, false> as Decode<'static>>::Error,
        ),
        /// while decoding alonzo style `Redeemers`
        Alonzo(
            #[from] <unique::codec::NonEmpty<super::legacy::Redeemer> as Decode<'static>>::Error,
        ),
    }

    impl Decode<'_> for Codec {
        type Error = Error;

        fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
            match d.datatype() {
                Ok(Type::Array | Type::ArrayIndef) => {
                    let non_empty: Unique<Vec<_>, false> =
                        unique::codec::NonEmpty::<super::legacy::Redeemer>::decode(d)?.into();
                    Ok(Codec(Unique(
                        non_empty
                            .0
                            .into_iter()
                            .map(
                                |super::legacy::Redeemer {
                                     kind,
                                     index,
                                     data,
                                     execution_units,
                                 }| {
                                    (
                                        super::Index { kind, index },
                                        super::Redeemer {
                                            data,
                                            execution_units,
                                        },
                                    )
                                },
                            )
                            .collect(),
                    )))
                }
                _ => Ok(Codec(Unique(
                    Unique::<Vec1<(super::Index, super::Redeemer)>, false>::decode(d)?
                        .0
                        .into_vec(),
                ))),
            }
        }
    }
}

mod legacy {
    use crate::alonzo::script::{Data, execution};
    use std::hash::Hash;

    #[repr(C)]
    #[derive(tinycbor_derive::Decode)]
    pub struct Redeemer {
        pub kind: super::index::Kind,
        pub index: u64,
        pub data: Data,
        pub execution_units: execution::Units,
    }

    impl PartialEq for Redeemer {
        fn eq(&self, other: &Self) -> bool {
            self.kind == other.kind && self.index == other.index
        }
    }

    impl Eq for Redeemer {}

    impl Hash for Redeemer {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.kind.hash(state);
            self.index.hash(state);
        }
    }
}
