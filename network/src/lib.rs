//! Implementation of the [network specification][net-spec] for the Cardano protocol.
//!
//! [net-spec]: https://ouroboros-network.cardano.intersectmbo.org/pdfs/network-spec/network-spec.pdf

pub mod mux;
pub mod protocol;
pub mod traits;
pub mod typefu;
pub mod hard_fork_combinator;

use std::{convert::Infallible, ops::Deref};

use minicbor::{CborLen, Decode, Encode};

#[doc(inline)]
pub use mux::mux;

#[repr(u32)]
#[derive(Debug, Encode, Decode)]
#[cbor(index_only)]
pub enum NetworkMagic {
    #[n(1)]
    Preprod = 1,
    #[n(2)]
    Preview = 2,
    #[n(764824073)]
    Mainnet = 764824073,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Point {
    Genesis,
    Block { slot: u64, hash: [u8; 32] },
}

impl<C> Encode<C> for Point {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        if let Point::Block { slot, hash } = self {
            e.array(2)?;
            e.u64(*slot)?;
            e.bytes(hash).map(|_| ())
        } else {
            e.array(0).map(|_| ())
        }
    }
}

impl<C> Decode<'_, C> for Point {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let len = d.array()?;
        if len == Some(0) {
            Ok(Point::Genesis)
        } else if len.is_none_or(|len| len == 2) {
            let slot = d.u64()?;
            let hash: minicbor::bytes::ByteArray<32> = d.decode()?;
            if len.is_none() {
                let ty = d.datatype()?;
                if ty != minicbor::data::Type::Break {
                    return Err(minicbor::decode::Error::type_mismatch(ty));
                }
                d.skip()?;
            }
            Ok(Point::Block {
                slot,
                hash: hash.into(),
            })
        } else {
            Err(minicbor::decode::Error::message("invalid array size"))
        }
    }
}

impl From<Tip> for Point {
    fn from(value: Tip) -> Self {
        match value {
            Tip::Genesis => Point::Genesis,
            Tip::Block { slot, hash, .. } => Point::Block { slot, hash },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tip {
    Genesis,
    Block {
        slot: u64,
        hash: [u8; 32],
        block_number: u64,
    },
}

impl<C> Encode<C> for Tip {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(2)?;
        let block_no = match self {
            Tip::Genesis => 0,
            Tip::Block { block_number, .. } => *block_number,
        };
        e.encode(Point::from(*self))?;
        e.u64(block_no).map(|_| ())
    }
}

impl<C> Decode<'_, C> for Tip {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        cbor_util::array_decode(2, |d| {
            let point = Point::decode(d, &mut ())?;
            let block_number = d.u64()?;
            Ok(match point {
                Point::Genesis => Tip::Genesis,
                Point::Block { slot, hash } => Tip::Block {
                    slot,
                    hash,
                    block_number,
                },
            })
        }, d)
    }
}

/// Keeps both the value and its encoded form.
///
/// It is bad practice to decode and then re-encode values when their hashes are computed based
/// on their encoding, because peers might encode values in a slightly different way that is
/// still considered valid. This would yield different hashes for the same data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WithEncoded<T> {
    value: T,
    encoded: Box<[u8]>,
}

impl<T: Encode<()>> WithEncoded<T> {
    /// Cache the encoding of the following value.
    pub fn new(value: T) -> Result<Self, minicbor::encode::Error<Infallible>> {
        let mut encoder = minicbor::Encoder::new(Vec::new());
        encoder.encode(&value)?;
        let encoded = encoder.into_writer().into_boxed_slice();
        Ok(WithEncoded { value, encoded })
    }
}

impl<T: for<'a> Decode<'a, ()>> WithEncoded<T> {
    /// Create a value from its encoding, and keep the original encoding.
    pub fn from_encoded(encoded: Box<[u8]>) -> Result<Self, minicbor::decode::Error> {
        let value: T = minicbor::decode(&encoded)?;
        Ok(Self { value, encoded })
    }
}

impl<T> WithEncoded<T> {
    /// Get the encoded bytes for this value.
    pub fn encoded(value: &WithEncoded<T>) -> &[u8] {
        &value.encoded
    }
}

impl<C, T: Encode<C>> Encode<C> for WithEncoded<T> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.writer_mut()
            .write_all(&self.encoded)
            .map_err(minicbor::encode::Error::write)
    }
}

impl<'a, C, T: Decode<'a, C>> Decode<'a, C> for WithEncoded<T> {
    fn decode(d: &mut minicbor::Decoder<'a>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        let start = d.position();
        let value = T::decode(d, ctx)?;
        let end = d.position();
        Ok(WithEncoded {
            value,
            encoded: d.input()[start..end].into(),
        })
    }
}

impl<C, T: CborLen<C>> CborLen<C> for WithEncoded<T> {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        self.value.cbor_len(ctx)
    }
}

impl<T> Deref for WithEncoded<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
