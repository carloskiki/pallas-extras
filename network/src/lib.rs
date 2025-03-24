pub mod block_fetch;
pub mod chain_sync;
pub mod handshake;
pub mod mux;

use minicbor::{Decode, Encode};
use zerocopy::transmute;

#[derive(Debug)]
pub struct Protocol<T> {
    pub responder: bool,
    pub protocol: T,
}

impl Protocol<NodeToNode> {
    pub fn from_u16(value: u16) -> Result<Self, UnknownProtocol> {
        let responder = value & (1 << 15) != 0;
        let value = value & !(1 << 15);

        let protocol = match value {
            0 => NodeToNode::Handshake,
            2 => NodeToNode::ChainSync,
            3 => NodeToNode::BlockFetch,
            4 => NodeToNode::TxSubmission,
            8 => NodeToNode::KeepAlive,
            10 => NodeToNode::PeerSharing,
            _ => return Err(UnknownProtocol),
        };

        Ok(Self {
            responder,
            protocol,
        })
    }

    pub fn to_u16(&self) -> u16 {
        let mut value = self.protocol as u16;
        if self.responder {
            value |= 1 << 15;
        }
        value
    }
}

impl Protocol<NodeToClient> {
    pub fn from_u16(value: u16) -> Result<Self, UnknownProtocol> {
        let responder = value & (1 << 15) != 0;
        let value = value & !(1 << 15);

        let protocol = match value {
            0 => NodeToClient::Handshake,
            5 => NodeToClient::ChainSync,
            6 => NodeToClient::LocalTxSubmission,
            7 => NodeToClient::LocalStateQuery,
            9 => NodeToClient::LocalTxMonitor,
            _ => return Err(UnknownProtocol),
        };

        Ok(Self {
            responder,
            protocol,
        })
    }

    pub fn to_u16(&self) -> u16 {
        let mut value = self.protocol as u16;
        if self.responder {
            value |= 1 << 15;
        }
        value
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NodeToNode {
    Handshake = 0,
    ChainSync = 2,
    BlockFetch = 3,
    TxSubmission = 4,
    KeepAlive = 8,
    PeerSharing = 10,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeToClient {
    Handshake = 0,
    ChainSync = 5,
    LocalTxSubmission = 6,
    LocalStateQuery = 7,
    LocalTxMonitor = 9,
}

#[derive(Debug)]
pub struct Header<T> {
    pub timestamp: u32,
    pub protocol: Protocol<T>,
    pub payload_len: u16,
}

impl TryFrom<[u8; 8]> for Header<NodeToNode> {
    type Error = UnknownProtocol;

    fn try_from(value: [u8; 8]) -> std::result::Result<Self, UnknownProtocol> {
        let [timestamp, rest]: [[u8; 4]; 2] = transmute!(value);
        let [protocol, payload_len]: [[u8; 2]; 2] = transmute!(rest);

        let timestamp = u32::from_be_bytes(timestamp);
        let protocol = Protocol::<NodeToNode>::from_u16(u16::from_be_bytes(protocol))?;
        let payload_len = u16::from_be_bytes(payload_len);

        Ok(Self {
            timestamp,
            protocol,
            payload_len,
        })
    }
}

impl TryFrom<[u8; 8]> for Header<NodeToClient> {
    type Error = UnknownProtocol;

    fn try_from(value: [u8; 8]) -> std::result::Result<Self, UnknownProtocol> {
        let [timestamp, rest]: [[u8; 4]; 2] = transmute!(value);
        let [protocol, payload_len]: [[u8; 2]; 2] = transmute!(rest);

        let timestamp = u32::from_be_bytes(timestamp);
        let protocol = Protocol::<NodeToClient>::from_u16(u16::from_be_bytes(protocol))?;
        let payload_len = u16::from_be_bytes(payload_len);

        Ok(Self {
            timestamp,
            protocol,
            payload_len,
        })
    }
}

impl From<Header<NodeToNode>> for [u8; 8] {
    fn from(value: Header<NodeToNode>) -> Self {
        let protocol_value = value.protocol.to_u16();
        let protocol_and_payload_len: [u8; 4] = transmute!([
            protocol_value.to_be_bytes(),
            value.payload_len.to_be_bytes()
        ]);
        let timestamp: [u8; 4] = value.timestamp.to_be_bytes();

        transmute!([timestamp, protocol_and_payload_len])
    }
}

impl From<Header<NodeToClient>> for [u8; 8] {
    fn from(value: Header<NodeToClient>) -> Self {
        let protocol_value = value.protocol.to_u16();
        let protocol_and_payload_len: [u8; 4] = transmute!([
            protocol_value.to_be_bytes(),
            value.payload_len.to_be_bytes()
        ]);
        let timestamp: [u8; 4] = value.timestamp.to_be_bytes();

        transmute!([timestamp, protocol_and_payload_len])
    }
}

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
pub struct UnknownProtocol;

impl std::fmt::Display for UnknownProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown protocol number")
    }
}

impl std::error::Error for UnknownProtocol {}

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
        let arr_len = d.array()?;
        if arr_len.is_some_and(|len| len != 2) {
            return Err(minicbor::decode::Error::message("invalid array size"));
        }
        let point = Point::decode(d, &mut ())?;
        let block_number = d.u64()?;
        if arr_len.is_none() {
            let ty = d.datatype()?;
            if  ty != minicbor::data::Type::Break {
                return Err(minicbor::decode::Error::type_mismatch(ty));
            }
            d.skip()?;
        }
        Ok(match point {
            Point::Genesis => Tip::Genesis,
            Point::Block { slot, hash } => Tip::Block {
                slot,
                hash,
                block_number,
            },
        })
    }
}

pub trait Request: Encode<()> {
    const TIMEOUT: Option<std::time::Duration>;
    type Response<'a>: Decode<'a, ()>;
}
