use tinycbor::{
    CborLen, Decode, Encode,
    collections::{self, fixed},
    tag,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Input<'a> {
    id: &'a super::Id,
    index: u32,
}

impl Encode for Input<'_> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        codec::Codec::from(*self).encode(e)
    }
}

impl<'a, 'b: 'a> Decode<'b> for Input<'a> {
    type Error = collections::Error<
        fixed::Error<
            tag::Error<tag::Error<collections::Error<collections::Error<fixed::Error<Error>>>>>,
        >,
    >;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        let codec::Codec::Input(codec::Inner { id, index }) =
            codec::Codec::decode(d).map_err(|e| {
                e.map(|e| {
                    e.map(|e| {
                        e.map(|e| match e {
                            codec::CodecError::Input(e) => e,
                        })
                    })
                })
            })?;
        Ok(Input { id, index })
    }
}

impl CborLen for Input<'_> {
    fn cbor_len(&self) -> usize {
        codec::Codec::from(*self).cbor_len()
    }
}

pub use codec::Error;

mod codec {
    use crate::byron::transaction;
    use tinycbor_derive::{CborLen, Decode, Encode};

    #[derive(Encode, Decode, CborLen)]
    pub(super) struct Inner<'a> {
        pub id: &'a transaction::Id,
        pub index: u32,
    }

    #[derive(Encode, Decode, CborLen)]
    #[cbor(error = "CodecError")]
    pub(super) enum Codec<'a> {
        #[n(0)]
        Input(#[cbor(with = "tinycbor::Encoded<Inner<'a>>")] Inner<'a>),
    }

    impl<'a> From<super::Input<'a>> for Codec<'a> {
        fn from(input: super::Input<'a>) -> Self {
            Codec::Input(Inner {
                id: input.id,
                index: input.index,
            })
        }
    }
}
