pub mod handshake;
pub mod chain_sync;

use minicbor::{Decode, Encode};
use zerocopy::transmute;

#[derive(Debug)]
pub struct Header {
    pub timestamp: u32,
    pub protocol: u16,
    pub payload_len: u16,
}

impl From<[u8; 8]> for Header {
    fn from(value: [u8; 8]) -> Self {
        let [timestamp, rest]: [[u8; 4]; 2] = transmute!(value);
        let [protocol, payload_len]: [[u8; 2]; 2] = transmute!(rest);

        let timestamp = u32::from_be_bytes(timestamp);
        let protocol = u16::from_be_bytes(protocol);
        let payload_len = u16::from_be_bytes(payload_len);

        Self {
            timestamp,
            protocol,
            payload_len,
        }
    }
}

impl From<Header> for [u8; 8] {
    fn from(value: Header) -> Self {
        let protocol_and_payload_len: [u8; 4] = transmute!([
            value.protocol.to_be_bytes(),
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

pub type Result<T, E> = std::result::Result<T, Error<E>>;

pub enum Error<E> {
    InvalidState,
    Timeout,
    ProtocolError(E),
}
