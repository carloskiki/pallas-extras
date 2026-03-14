use crate::{allegra, alonzo, babbage, byron, conway, mary, shelley};
use tinycbor::{
    CborLen, Decode, Encode,
    container::{self, bounded},
    tag,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Header<'a> {
    Boundary(byron::block::boundary::Header<'a>),
    Byron(byron::block::Header<'a>),
    Shelley(shelley::block::Header<'a>),
    Allegra(allegra::block::Header<'a>),
    Mary(mary::block::Header<'a>),
    Alonzo(alonzo::block::Header<'a>),
    Babbage(babbage::block::Header<'a>),
    Conway(conway::block::Header<'a>),
}

impl Encode for Header<'_> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        e.array(2)?;
        match self {
            Header::Boundary(header) => {
                0.encode(e)?;
                e.array(2)?;
                e.array(2)?;
                0.encode(e)?;
                0.encode(e)?; // This is "block size hint". We don't expect to ever encode byron
                // era for consensus purposes, so we just use 0.
                tinycbor::Encoded(header).encode(e)
            }
            Header::Byron(header) => {
                0.encode(e)?;
                e.array(2)?;
                e.array(2)?;
                1.encode(e)?;
                0.encode(e)?; // This is "block size hint". We don't expect to ever encode byron
                // era for consensus purposes, so we just use 0.
                tinycbor::Encoded(header).encode(e)
            }
            Header::Shelley(header) => {
                1.encode(e)?;
                tinycbor::Encoded(header).encode(e)
            }
            Header::Allegra(header) => {
                2.encode(e)?;
                tinycbor::Encoded(header).encode(e)
            }
            Header::Mary(header) => {
                3.encode(e)?;
                tinycbor::Encoded(header).encode(e)
            }
            Header::Alonzo(header) => {
                4.encode(e)?;
                tinycbor::Encoded(header).encode(e)
            }
            Header::Babbage(header) => {
                5.encode(e)?;
                tinycbor::Encoded(header).encode(e)
            }
            Header::Conway(header) => {
                6.encode(e)?;
                tinycbor::Encoded(header).encode(e)
            }
        }
    }
}

impl<'a, 'b: 'a> Decode<'b> for Header<'a> {
    type Error = container::Error<bounded::Error<tag::Error<codec::Error>>>;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        let codec = codec::Codec::decode(d)?;
        Ok(match codec {
            codec::Codec::Byron(byron) => match byron {
                codec::ByronCodec::Boundary(header) => Header::Boundary(header),
                codec::ByronCodec::Byron(header) => Header::Byron(header),
            },
            codec::Codec::Shelley(header) => Header::Shelley(header),
            codec::Codec::Allegra(header) => Header::Allegra(header),
            codec::Codec::Mary(header) => Header::Mary(header),
            codec::Codec::Alonzo(header) => Header::Alonzo(header),
            codec::Codec::Babbage(header) => Header::Babbage(header),
            codec::Codec::Conway(header) => Header::Conway(header),
        })
    }
}

impl CborLen for Header<'_> {
    fn cbor_len(&self) -> usize {
        2.cbor_len()
            + match self {
                Header::Boundary(header) => {
                    0.cbor_len()
                        + 2.cbor_len()
                        + 2.cbor_len()
                        + 0.cbor_len()
                        + 0.cbor_len()
                        + tinycbor::Encoded(header).cbor_len()
                }
                Header::Byron(header) => {
                    0.cbor_len()
                        + 2.cbor_len()
                        + 2.cbor_len()
                        + 1.cbor_len()
                        + 0.cbor_len()
                        + tinycbor::Encoded(header).cbor_len()
                }
                Header::Shelley(header) => 1.cbor_len() + tinycbor::Encoded(header).cbor_len(),
                Header::Allegra(header) => 2.cbor_len() + tinycbor::Encoded(header).cbor_len(),
                Header::Mary(header) => 3.cbor_len() + tinycbor::Encoded(header).cbor_len(),
                Header::Alonzo(header) => 4.cbor_len() + tinycbor::Encoded(header).cbor_len(),
                Header::Babbage(header) => 5.cbor_len() + tinycbor::Encoded(header).cbor_len(),
                Header::Conway(header) => 6.cbor_len() + tinycbor::Encoded(header).cbor_len(),
            }
    }
}

mod codec {
    use crate::{allegra, alonzo, babbage, byron, conway, mary, shelley};
    use tinycbor::{
        Decode,
        container::{self, bounded},
        primitive, tag,
    };
    use tinycbor_derive::Decode;

    #[derive(Debug, displaydoc::Display, thiserror::Error)]
    enum ByronError {
        /// while decoding a byron era boundary block header.
        Boundary(
            #[source]
            <tinycbor::Encoded<byron::block::boundary::Header<'static>> as Decode<'static>>::Error,
        ),
        /// while decoding a byron era block header.
        Byron(
            #[source] <tinycbor::Encoded<byron::block::Header<'static>> as Decode<'static>>::Error,
        ),
    }

    pub enum ByronCodec<'a> {
        Boundary(byron::block::boundary::Header<'a>),
        Byron(byron::block::Header<'a>),
    }

    impl<'a, 'b: 'a> Decode<'b> for ByronCodec<'a> {
        type Error = container::Error<bounded::Error<tag::Error<ByronError>>>;

        fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
            #[derive(Decode)]
            #[cbor(error = "IndexError")]
            struct Index {
                tag: u64,
                block_size_hint: u32,
            }

            let mut visitor = d.array_visitor()?;
            let index = visitor
                .visit::<Index>()
                .ok_or(bounded::Error::Missing)?
                .map_err(|e| match e {
                    container::Error::Malformed(primitive::Error::EndOfInput) => {
                        tag::Error::Malformed(primitive::Error::EndOfInput)
                    }
                    _ => tag::Error::Malformed(primitive::Error::InvalidHeader),
                })
                .map_err(|e| container::Error::Content(bounded::Error::Content(e)))?;
            match index.tag {
                0 => {
                    let header = tinycbor::Encoded::<byron::block::boundary::Header>::decode(d)
                        .map_err(|e| {
                            container::Error::Content(bounded::Error::Content(tag::Error::Content(
                                ByronError::Boundary(e),
                            )))
                        })?;
                    Ok(ByronCodec::Boundary(header.0))
                }
                1 => {
                    let header =
                        tinycbor::Encoded::<byron::block::Header>::decode(d).map_err(|e| {
                            container::Error::Content(bounded::Error::Content(tag::Error::Content(
                                ByronError::Byron(e),
                            )))
                        })?;
                    Ok(ByronCodec::Byron(header.0))
                }
                _ => Err(container::Error::Malformed(primitive::Error::InvalidHeader)),
            }
        }
    }

    #[derive(Decode)]
    pub enum Codec<'a> {
        #[n(0)]
        Byron(ByronCodec<'a>),
        #[n(1)]
        Shelley(
            #[cbor(with = "tinycbor::Encoded<shelley::block::Header<'a>>")]
            shelley::block::Header<'a>,
        ),
        #[n(2)]
        Allegra(
            #[cbor(with = "tinycbor::Encoded<allegra::block::Header<'a>>")]
            allegra::block::Header<'a>,
        ),
        #[n(3)]
        Mary(#[cbor(with = "tinycbor::Encoded<mary::block::Header<'a>>")] mary::block::Header<'a>),
        #[n(4)]
        Alonzo(
            #[cbor(with = "tinycbor::Encoded<alonzo::block::Header<'a>>")]
            alonzo::block::Header<'a>,
        ),
        #[n(5)]
        Babbage(
            #[cbor(with = "tinycbor::Encoded<babbage::block::Header<'a>>")]
            babbage::block::Header<'a>,
        ),
        #[n(6)]
        Conway(
            #[cbor(with = "tinycbor::Encoded<conway::block::Header<'a>>")]
            conway::block::Header<'a>,
        ),
    }
}
