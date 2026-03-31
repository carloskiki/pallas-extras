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
use tinycbor::{Encode, Encoder};

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

pub(crate) struct Egress(BytesMut);

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
                while buf.len() != 0 {
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

        let mut encoder = Encoder(Writer(buffer, 0, protocol));
        encoder.begin_array();
        M::TAG.encode(&mut encoder);
        message.encode(&mut encoder);
        encoder.end();

        let message = buffer.split();
        Egress(message)
    }

    /// Write header data to the message.
    pub fn finalize(mut self, timestamp: Timestamp) -> Bytes {
        const HEADER_SIZE: usize = std::mem::size_of::<Header>();

        self.0
            .chunks_mut(u16::MAX as usize + HEADER_SIZE)
            .for_each(|chunk| {
                let chunk_len = chunk.len() - HEADER_SIZE;
                let header_array: &mut [u8; HEADER_SIZE] =
                    &mut chunk[..HEADER_SIZE].try_into().expect("sizes match");
                let header: &mut Header = zerocopy::transmute_mut!(header_array);
                header.payload_len = (chunk_len as u16).into();
                header.timestamp = timestamp;
            });

        self.0.freeze()
    }
}

pub(crate) struct Ingress {
    message: Bytes,
    timestamp: Timestamp,
}
