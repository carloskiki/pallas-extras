use tinycbor::{CborLen, Decode, Encode};
use tinycbor_derive::{CborLen, Decode, Encode};

use crate::point::Point;

/// The tip of the block chain.
///
/// Some mini-protocols require this information in responses, indicating the current state of the
/// block chain as seen by the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tip {
    /// The genesis block.
    Genesis,
    /// A specific block on the chain.
    Block {
        /// The block's slot number.
        slot: u64,
        /// The hash of the block header.
        hash: [u8; 32],
        /// The block number.
        block_number: u64,
    },
}

impl Tip {
    fn to_codec(&self) -> Codec {
        match self {
            Tip::Genesis => Codec {
                block_number: 0,
                point: Point::Genesis,
            },
            Tip::Block {
                slot,
                hash,
                block_number,
            } => Codec {
                block_number,
                point: Point::Block { slot, hash },
            },
        }
    }
}

impl Encode for Tip {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        self.to_codec().encode(e)
    }
}

impl Decode<'_> for Tip {
    type Error = Error;

    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        // We don't ensure that the `block_number` of  `Genesis` is `0`.
        Codec::decode(d).into()
    }
}

impl CborLen for Tip {
    fn cbor_len(&self) -> usize {
        self.to_codec().cbor_len()
    }
}

#[derive(Decode, Encode, CborLen)]
struct Codec {
    block_number: ledger::shelley::block::Number,
    point: Point,
}

impl From<Codec> for Tip {
    fn from(
        Codec {
            block_number,
            point,
        }: Codec,
    ) -> Self {
        match point {
            Point::Genesis => Tip::Genesis,
            Point::Block { slot, hash } => Tip::Block {
                slot,
                hash,
                block_number,
            },
        }
    }
}
