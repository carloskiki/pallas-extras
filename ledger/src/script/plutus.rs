use minicbor::{CborLen, Decode, Encode};
use num_bigint::BigInt;

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Encode, Decode, CborLen)]
#[cbor(transparent)]
pub struct Script(#[cbor(with = "minicbor::bytes")] Box<[u8]>);

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Data {
    Map(Box<[(Data, Data)]>),
    List(Box<[Data]>),
    Bytes(Box<[u8]>),
    BigInt(BigInt),
    Construct(Construct),
}

impl<C> Encode<C> for Data {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Data::Map(items) => cbor_util::list_as_map::encode(items, e, ctx),
            Data::List(data) => e.encode(data)?.ok(),
            Data::Bytes(bytes) => cbor_util::bounded_bytes::encode(bytes, e, ctx),
            Data::BigInt(big_int) if big_int.iter_u64_digits().count() < 2 => e
                .encode(
                    minicbor::data::Int::try_from(
                        i128::try_from(big_int).expect("should fit since only one u64 digit"),
                    )
                    .expect("should fit since only one u64 digit"),
                )?
                .ok(),
            Data::BigInt(big_int) => {
                e.tag(match big_int.sign() {
                    num_bigint::Sign::Minus => minicbor::data::IanaTag::NegBignum,
                    num_bigint::Sign::Plus => minicbor::data::IanaTag::PosBignum,
                    _ => {
                        unreachable!("value should not be zero, it is matched in the previous arm")
                    }
                })?;
                cbor_util::bounded_bytes::encode(
                    &(big_int
                        + if big_int.sign() == num_bigint::Sign::Minus {
                            1u8
                        } else {
                            0
                        })
                    .to_bytes_be()
                    .1,
                    e,
                    ctx,
                )
            }
            Data::Construct(construct) => e.encode(construct)?.ok(),
        }
    }
}

impl<C> Decode<'_, C> for Data {
    fn decode(d: &mut minicbor::Decoder<'_>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::U8
            | minicbor::data::Type::U16
            | minicbor::data::Type::U32
            | minicbor::data::Type::U64
            | minicbor::data::Type::I8
            | minicbor::data::Type::I16
            | minicbor::data::Type::I32
            | minicbor::data::Type::I64
            | minicbor::data::Type::Int => {
                let int: i128 = d.int()?.into();
                Ok(Self::BigInt(BigInt::from(int)))
            }
            minicbor::data::Type::Bytes | minicbor::data::Type::BytesIndef => {
                Ok(Self::Bytes(cbor_util::bounded_bytes::decode(d, ctx)?))
            }
            minicbor::data::Type::Array | minicbor::data::Type::ArrayIndef => {
                Ok(Self::List(cbor_util::boxed_slice::decode(d, ctx)?))
            }
            minicbor::data::Type::Map | minicbor::data::Type::MapIndef => {
                Ok(Self::Map(cbor_util::list_as_map::decode(d, ctx)?))
            }
            minicbor::data::Type::Tag => {
                let pre_tag = d.position();
                match d.tag()?.as_u64() {
                    2 => Ok(Self::BigInt(BigInt::from_bytes_be(
                        num_bigint::Sign::Plus,
                        &cbor_util::bounded_bytes::decode(d, ctx)?,
                    ))),
                    3 => Ok(Self::BigInt(
                        BigInt::from_bytes_be(
                            num_bigint::Sign::Minus,
                            &cbor_util::bounded_bytes::decode(d, ctx)?,
                        ) - 1u8,
                    )),
                    102 | 121..=127 | 1280..=1400 => {
                        d.set_position(pre_tag);
                        Ok(Self::Construct(d.decode()?))
                    }
                    t => Err(
                        minicbor::decode::Error::tag_mismatch(minicbor::data::Tag::new(t))
                            .at(d.position()),
                    ),
                }
            }
            t => Err(minicbor::decode::Error::type_mismatch(t).at(d.position())),
        }
    }
}

impl<C> CborLen<C> for Data {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        match self {
            Data::Map(items) => cbor_util::list_as_map::cbor_len(items, ctx),
            Data::List(datas) => datas.cbor_len(ctx),
            Data::Bytes(items) => cbor_util::bounded_bytes::cbor_len(items, ctx),
            Data::BigInt(big_int) if big_int.iter_u64_digits().count() < 2 => {
                minicbor::data::Int::try_from(
                    i128::try_from(big_int).expect("should fit since only one u64 digit"),
                )
                .expect("should fit since only one u64 digit")
                .cbor_len(ctx)
            }
            Data::BigInt(big_int) => {
                let len = match big_int.sign() {
                    num_bigint::Sign::Minus => (big_int + 1u8).bits().div_ceil(8),
                    num_bigint::Sign::Plus => big_int.bits().div_ceil(8),
                    num_bigint::Sign::NoSign => {
                        unreachable!(
                            "value should not be zero, as it is matched in the previous arm"
                        )
                    }
                } as usize;
                // Tag size + size of bounded bytes with this len.
                1 + if len <= 64 {
                    len.cbor_len(ctx) + len
                } else {
                    let last_chunk_len = len % 64;
                    2 + (len / 64) * (64.cbor_len(ctx) + 64)
                        + if last_chunk_len != 0 {
                            last_chunk_len.cbor_len(ctx) + last_chunk_len
                        } else {
                            0
                        }
                }
            }
            Data::Construct(construct) => construct.cbor_len(ctx),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Construct {
    pub tag: u64,
    pub value: Box<[Data]>,
}

impl<C> Encode<C> for Construct {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        if self.tag <= 6 {
            e.tag(minicbor::data::Tag::new(self.tag + 121))?;
        } else if self.tag <= 127 {
            e.tag(minicbor::data::Tag::new(self.tag + 1280 - 7))?;
        } else {
            e.tag(minicbor::data::Tag::new(102))?
                .array(2)?
                .u64(self.tag)?;
        }
        e.encode(&self.value)?;
        Ok(())
    }
}

impl<C> Decode<'_, C> for Construct {
    fn decode(d: &mut minicbor::Decoder<'_>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        let tag = d.tag()?.as_u64();
        match tag {
            102 => {
                #[derive(Decode)]
                struct Inner {
                    #[n(0)]
                    tag: u64,
                    #[cbor(n(1), with = "cbor_util::boxed_slice")]
                    value: Box<[Data]>,
                }
                let Inner { tag, value } = d.decode()?;
                Ok(Self { tag, value })
            }

            121..=127 => Ok(Construct {
                tag: tag - 121,
                value: cbor_util::boxed_slice::decode(d, ctx)?,
            }),
            1280..=1400 => Ok(Construct {
                tag: tag - 1280 + 7,
                value: cbor_util::boxed_slice::decode(d, ctx)?,
            }),
            t => Err(
                minicbor::decode::Error::tag_mismatch(minicbor::data::Tag::new(t)).at(d.position()),
            ),
        }
    }
}

impl<C> CborLen<C> for Construct {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        if self.tag <= 6 {
            minicbor::data::Tag::new(self.tag + 121).cbor_len(ctx) + self.value.cbor_len(ctx)
        } else if self.tag <= 127 {
            minicbor::data::Tag::new(self.tag + 1280 - 7).cbor_len(ctx) + self.value.cbor_len(ctx)
        } else {
            // self.tag <= 127, because of constructor
            minicbor::data::Tag::new(102).cbor_len(ctx)
                + 2.cbor_len(ctx)
                + self.tag.cbor_len(ctx)
                + self.value.cbor_len(ctx)
        }
    }
}
