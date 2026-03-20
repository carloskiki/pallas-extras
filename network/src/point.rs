use displaydoc::Display;
use thiserror::Error;
use tinycbor::{
    CborLen, Decode, Encode,
    container::{self, bounded},
    primitive,
};

use crate::Tip;

/// A point on the block chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Point {
    /// The genesis block.
    Genesis,
    /// A specific block on the chain.
    Block {
        /// The slot number of the block.
        slot: u64,
        /// The hash of the block header.
        ///
        /// This is used to distinguish blocks and epoch boundary blocks (EBBs) in the Byron era.
        hash: [u8; 32],
    },
}

#[derive(Debug, Display, Error)]
pub enum Error {
    /// while decoding the point's slot
    BlockSlot(#[from] primitive::Error),
    /// while decoding the point's hash
    BlockHash(#[from] container::Error<bounded::Error<std::convert::Infallible>>),
}

impl Encode for Point {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        if let Point::Block { slot, hash } = self {
            e.array(2)?;
            slot.encode(e)?;
            hash.encode(e)
        } else {
            e.array(0)
        }
    }
}

impl Decode<'_> for Point {
    type Error = container::Error<bounded::Error<Error>>;

    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        let mut visitor = d.array_visitor()?;
        if visitor.remaining() == Some(0) {
            return Ok(Point::Genesis);
        }
        let ret = Self::Block {
            slot: visitor
                .visit()
                .ok_or(bounded::Error::Missing)?
                .map_err(|e| bounded::Error::Content(Error::BlockSlot(e)))?,
            hash: visitor
                .visit()
                .ok_or(bounded::Error::Missing)?
                .map_err(|e| bounded::Error::Content(Error::BlockHash(e)))?,
        };
        if visitor.remaining() != Some(0) {
            return Err(bounded::Error::Surplus.into());
        }
        Ok(ret)
    }
}

impl CborLen for Point {
    fn cbor_len(&self) -> usize {
        if let Point::Block { slot, hash } = self {
            2.cbor_len() + slot.cbor_len() + hash.cbor_len()
        } else {
            0.cbor_len()
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
