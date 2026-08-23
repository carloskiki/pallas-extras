use blake2::{Blake2b256, Digest as _};
use bytes::Bytes;

pub use network::{Point, Tip};

/// A Blake2b-256 digest.
pub type Digest = [u8; 32];

/// Metadata and original bytes for a validated Cardano block header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub point: Point,
    pub parent: Option<Digest>,
    pub block_number: u64,
    pub cbor: Bytes,
}

impl Header {
    /// Construct a header and verify that its bytes produce the advertised point.
    pub fn new(
        point: Point,
        parent: Option<Digest>,
        block_number: u64,
        cbor: impl Into<Bytes>,
    ) -> Result<Self, ValidationError> {
        let cbor = cbor.into();
        let Point::Block { hash, .. } = point else {
            return Err(ValidationError::GenesisHeader);
        };
        if digest(&cbor) != hash {
            return Err(ValidationError::HeaderHash);
        }
        Ok(Self {
            point,
            parent,
            block_number,
            cbor,
        })
    }

    /// Build deterministic simulation data with the same hash/link validation as live headers.
    pub fn synthetic(parent: Point, block_number: u64, slot: u64) -> Self {
        let mut cbor = Vec::with_capacity(80);
        cbor.extend_from_slice(b"pallas-extras-synthetic-header-v1");
        cbor.extend_from_slice(&block_number.to_be_bytes());
        cbor.extend_from_slice(&slot.to_be_bytes());
        match parent {
            Point::Genesis => cbor.push(0),
            Point::Block {
                slot: parent_slot,
                hash,
            } => {
                cbor.push(1);
                cbor.extend_from_slice(&parent_slot.to_be_bytes());
                cbor.extend_from_slice(&hash);
            }
        }
        let parent_hash = match parent {
            Point::Genesis => None,
            Point::Block { hash, .. } => Some(hash),
        };
        let hash = digest(&cbor);
        Self {
            point: Point::Block { slot, hash },
            parent: parent_hash,
            block_number,
            cbor: Bytes::from(cbor),
        }
    }

    pub fn slot(&self) -> u64 {
        match self.point {
            Point::Genesis => 0,
            Point::Block { slot, .. } => slot,
        }
    }

    pub fn verify(&self) -> Result<(), ValidationError> {
        let Point::Block { hash, .. } = self.point else {
            return Err(ValidationError::GenesisHeader);
        };
        if digest(&self.cbor) == hash {
            Ok(())
        } else {
            Err(ValidationError::HeaderHash)
        }
    }
}

/// A full block associated with a chain-sync header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub header: Header,
    pub cbor: Bytes,
}

impl Block {
    pub fn synthetic(header: Header, payload_bytes: usize) -> Self {
        let mut cbor = Vec::with_capacity(payload_bytes.max(40));
        cbor.extend_from_slice(b"pallas-extras-synthetic-block-v1");
        cbor.extend_from_slice(&header.block_number.to_be_bytes());
        cbor.resize(payload_bytes.max(cbor.len()), header.block_number as u8);
        Self {
            header,
            cbor: Bytes::from(cbor),
        }
    }

    pub fn verify_against(&self, expected: &Header) -> Result<(), ValidationError> {
        self.header.verify()?;
        if self.header.point != expected.point
            || self.header.parent != expected.parent
            || self.header.block_number != expected.block_number
        {
            return Err(ValidationError::UnexpectedBlock);
        }
        if self.cbor.is_empty() {
            return Err(ValidationError::EmptyBlock);
        }
        Ok(())
    }
}

/// A change emitted by the chain synchronizer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChainEvent {
    RollForward { header: Header, tip: Tip },
    RollBackward { point: Point, tip: Tip },
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    #[error("a block header cannot use the genesis point")]
    GenesisHeader,
    #[error("header CBOR does not match its Blake2b-256 id")]
    HeaderHash,
    #[error("header does not extend the current chain")]
    Parent,
    #[error("block number does not extend the current chain")]
    BlockNumber,
    #[error("slot does not increase")]
    Slot,
    #[error("rollback point is not present in the selected chain")]
    UnknownRollback,
    #[error("fetched block does not match the selected header")]
    UnexpectedBlock,
    #[error("fetched block is empty")]
    EmptyBlock,
}

pub fn digest(bytes: &[u8]) -> Digest {
    Blake2b256::digest(bytes).into()
}

pub(crate) fn point_hash(point: Point) -> Option<Digest> {
    match point {
        Point::Genesis => None,
        Point::Block { hash, .. } => Some(hash),
    }
}
