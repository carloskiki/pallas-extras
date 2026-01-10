use tinycbor::{
    CborLen, Decode, Encode,
    collections::{self, fixed},
    tag,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FeePolicy {
    pub constant: u64,
    pub coefficient: u64,
}

impl Encode for FeePolicy {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        codec::Codec::from(*self).encode(e)
    }
}

impl<'a> Decode<'a> for FeePolicy {
    type Error = collections::Error<
        fixed::Error<
            tag::Error<tag::Error<collections::Error<collections::Error<fixed::Error<Error>>>>>,
        >,
    >;

    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        match codec::Codec::decode(d).map_err(|e| {
            e.map(|e| {
                e.map(|e| {
                    e.map(|e| match e {
                        codec::CodecError::Content(e) => e,
                    })
                })
            })
        })? {
            codec::Codec::Content(inner) => Ok(FeePolicy {
                constant: inner.constant,
                coefficient: inner.coefficient,
            }),
        }
    }
}

impl CborLen for FeePolicy {
    fn cbor_len(&self) -> usize {
        codec::Codec::from(*self).cbor_len()
    }
}

pub use codec::Error;

mod codec {
    use tinycbor_derive::{CborLen, Decode, Encode};

    #[derive(Encode, Decode, CborLen)]
    pub(super) struct Inner {
        pub constant: u64,
        pub coefficient: u64,
    }

    #[derive(Encode, Decode, CborLen)]
    #[cbor(error = "CodecError")]
    pub(super) enum Codec {
        #[n(0)]
        Content(#[cbor(with = "tinycbor::Encoded<Inner>")] Inner),
    }

    impl From<super::FeePolicy> for Codec {
        fn from(value: super::FeePolicy) -> Self {
            Codec::Content(Inner {
                constant: value.constant,
                coefficient: value.coefficient,
            })
        }
    }
}
