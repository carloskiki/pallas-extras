//! Implementation of the Multiplexer.
//!
//! The type requirements for the [`mux`] function may seem daunting, but the function's
//! documentation is quite clear.

use crate::{
    Message,
    mux::header::{ProtocolNumber, Timestamp},
};
use bytes::{Bytes, BytesMut};
use std::io;
use tinycbor::{Decode, Encode, Encoder, Write};

// TODO: In client and server, ensure that the timeouts are checked.
// TODO: Check for cancel safety anywhere `select!` is used.
// TODO: Check for snoozing (pretty much anywhere async is used).

pub mod handle;
pub use handle::Handle;

pub mod header;
pub use header::Header;
pub(crate) mod task;

/// Errors that can occur while using the multiplexer.
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum MuxError {
    /// IO error
    Io(#[from] io::Error),
    /// receive message for an unknown protocol
    UnknownProtocol(ProtocolNumber),
    /// received malformed message from a peer
    Malformed(Bytes),
    /// received a message that does not conform to protocol agency
    UnexpectedMessage(Bytes),
    /// receiving buffer for protocol {0:?} is full
    Full(ProtocolNumber),
    /// all handles have been dropped
    Closed,
}

#[doc(hidden)]
pub struct Egress(BytesMut);

impl Egress {
    pub fn new<M: Message + Encode>(
        message: &M,
        buffer: &mut BytesMut,
        protocol: ProtocolNumber,
    ) -> Self {
        /// Adapter to allow encoding into a `BytesMut`, and limiting messages to the maximum multiplexer
        /// message size.
        struct Writer<'a>(&'a mut bytes::BytesMut, usize, ProtocolNumber);

        impl embedded_io::ErrorType for Writer<'_> {
            type Error = std::convert::Infallible;
        }

        impl tinycbor::Write for Writer<'_> {
            fn write(&mut self, mut buf: &[u8]) -> Result<usize, Self::Error> {
                let written = buf.len();
                while !buf.is_empty() {
                    if self.1 == 0 {
                        let header = Header {
                            protocol: self.2,
                            timestamp: Default::default(),
                            payload_len: Default::default(),
                        };
                        self.0.extend_from_slice(zerocopy::transmute_ref!(&header));
                        self.1 = u16::MAX as usize;
                    }

                    let to_write = std::cmp::min(buf.len(), self.1);
                    self.0.extend_from_slice(&buf[..to_write]);
                    buf = &buf[to_write..];
                    self.1 -= to_write;
                }

                Ok(written)
            }

            fn flush(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }
        }

        let payload = tinycbor::to_vec(message);
        let mut decoder = tinycbor::Decoder(&payload);
        let mut payload_items = 0;
        while !decoder.0.is_empty() {
            tinycbor::Any::decode(&mut decoder).expect("Encode produced valid CBOR");
            payload_items += 1;
        }

        let mut encoder = Encoder(Writer(buffer, 0, protocol));
        encoder.array(1 + payload_items);
        M::TAG.encode(&mut encoder);
        encoder
            .0
            .write_all(&payload)
            .expect("multiplexer writer is infallible");

        let message = buffer.split();
        Egress(message)
    }

    /// Write header data to the message.
    pub fn finalize(mut self, timestamp: Timestamp) -> Bytes {
        const HEADER_SIZE: usize = std::mem::size_of::<Header>();

        self.0
            .as_mut()
            .chunks_mut(u16::MAX as usize + HEADER_SIZE)
            .for_each(|chunk| {
                let chunk_len = chunk.len() - HEADER_SIZE;
                let header_array: &mut [u8; HEADER_SIZE] =
                    (&mut chunk[..HEADER_SIZE]).try_into().expect("sizes match");
                let header: &mut Header = zerocopy::transmute_mut!(header_array);
                header.payload_len = (chunk_len as u16).into();
                header.timestamp = timestamp;
            });

        self.0.freeze()
    }
}

pub(crate) struct Ingress {
    message: Bytes,
}

/// Start a multiplexer over an asynchronous bearer.
///
/// The returned handles represent both sides of every mini-protocol in `P`. The task handle
/// resolves only when the bearer closes or a protocol error occurs.
pub fn mux<P, B>(bearer: B) -> (P::Handles, tokio::task::JoinHandle<MuxError>)
where
    P: crate::Protocol + Send + 'static,
    P::State: Send + 'static,
    P::Handles: Send + 'static,
    B: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (sender, receiver) = tokio::sync::mpsc::channel(64);
    let (handles, state) = P::initialize(sender);
    let task = tokio::spawn(task::task::<P>(bearer, receiver, state));
    (handles, task)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_to_node::keep_alive::KeepAlive;

    #[test]
    fn egress_starts_with_a_complete_mux_header() {
        let mut buffer = BytesMut::new();
        let framed = Egress::new(
            &KeepAlive { cookie: 42 },
            &mut buffer,
            ProtocolNumber::new(8, false),
        )
        .finalize(Timestamp::default());
        let bytes: &[u8; 8] = framed[..8].try_into().unwrap();
        let header: &Header = zerocopy::transmute_ref!(bytes);
        assert_eq!(header.protocol.number(), 8);
        assert_eq!(header.payload_len.get() as usize, framed.len() - 8);
    }

    #[test]
    fn chain_sync_find_intersect_matches_the_wire_codec() {
        let mut buffer = BytesMut::new();
        let framed = Egress::new(
            &crate::node_to_node::chain_sync::idle::FindIntersect {
                points: vec![crate::Point::Genesis],
            },
            &mut buffer,
            ProtocolNumber::new(2, false),
        )
        .finalize(Timestamp::default());

        // [MsgFindIntersect = 4, [Origin]]
        assert_eq!(&framed[8..], &[0x82, 0x04, 0x81, 0x80]);
    }

    #[test]
    fn zero_payload_messages_use_a_single_item_array() {
        let mut buffer = BytesMut::new();
        let framed = Egress::new(
            &crate::node_to_node::chain_sync::idle::Next,
            &mut buffer,
            ProtocolNumber::new(2, false),
        )
        .finalize(Timestamp::default());

        assert_eq!(&framed[8..], &[0x81, 0x00]);
    }

    #[test]
    fn handshake_proposal_uses_a_version_map() {
        let mut buffer = BytesMut::new();
        let framed = Egress::new(
            &crate::handshake::propose::Versions(crate::handshake::VersionTable {
                versions: vec![(
                    14,
                    crate::node_to_node::VersionData {
                        network_magic: crate::NetworkMagic::Preprod,
                        diffusion_mode: true,
                        peer_sharing: false,
                        query: false,
                    },
                )],
            }),
            &mut buffer,
            ProtocolNumber::new(0, false),
        )
        .finalize(Timestamp::default());

        assert_eq!(
            &framed[8..],
            &[0x82, 0x00, 0xa1, 0x0e, 0x84, 0x01, 0xf5, 0x00, 0xf4]
        );
    }
}
