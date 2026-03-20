use crate::{
    mux::{
        Egress, Ingress, MuxError,
        header::{Header, ProtocolNumber},
    },
    traits::protocol::Protocol,
    typefu::array::Index,
};
use bytes::{BufMut, BytesMut};
use hybrid_array::Array;
use std::collections::VecDeque;
use tinycbor::Decoder;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    select,
    sync::mpsc::{Receiver, Sender, error::TrySendError},
};

pub(super) async fn task<P>(
    mut bearer: impl AsyncRead + AsyncWrite + Unpin,
    mut receiver: Receiver<Egress<P>>,
    mut state: Array<MiniProtocolState, P::Length>,
) -> MuxError<P>
where
    P: Protocol + Index,
{
    let time = std::time::Instant::now();
    let mut reader_task = ReaderTask {
        header: Err(([0; _], 0)),
    };

    loop {
        select! {
            request = receiver.recv() => {
                let Some(request) = request else {
                    return MuxError::Closed;
                };

                if let Err(e) = writer_task(
                    &mut bearer,
                    request,
                    &time,
                    &mut state,
                ).await {
                    return e;
                }
            },
            result = reader_task.read_message(&mut bearer, &mut state) => {
                if let Err(e) = result {
                    return e;
                }
            }
        }
    }
}

async fn writer_task<P: Protocol + Index>(
    writer: &mut (impl AsyncWrite + Unpin),
    Egress {
        mut message,
        protocol,
        send_back,
    }: Egress<P>,
    time: &std::time::Instant,
    state: &mut Array<MiniProtocolState, P::Length>,
) -> Result<(), MuxError<P>> {
    // Write the timestamp to the header. TODO: this should be a wrapper struct over the buffer to
    // better document the behavior and ensure correctness.
    message.copy_from_slice(&(time.elapsed().as_micros() as u32).to_be_bytes());
    // 1. Send the bytes
    // TODO: buffer pool. It needs to maintain space for the timestamp in the header.
    writer.write_all(&message).await?;
    // 2. Add write back to state.
    if let Some(client_send_back) = send_back {
        let task_state = &mut state[protocol.index()];
        task_state.client_send_backs.push_back(client_send_back);
    }

    Ok(())
}

struct MiniProtocolState {
    read_buffer: BytesMut,
    read_state: tinycbor::stream::Any,
    server_send_back: Sender<Ingress>,
    client_send_backs: VecDeque<Sender<Ingress>>,
}

struct ReaderTask<P: Protocol + Index> {
    header: Result<Header<P>, ([u8; 8], u8)>,
}

impl<P: Protocol + Index> ReaderTask<P> {
    /// Read a message from the reader and return it.
    ///
    /// The future returned by this method is cancel safe.
    async fn read_message(
        &mut self,
        reader: &mut (impl AsyncRead + Unpin),
        state: &mut Array<MiniProtocolState, P::Length>,
    ) -> Result<(), MuxError<P>> {
        let Header {
            timestamp,
            protocol:
                ProtocolNumber {
                    protocol,
                    server_sent,
                },
            payload_len: remaining,
        } = match &mut self.header {
            Ok(header) => header,
            Err((buffer, cursor)) => {
                while *cursor != 8 {
                    let read = reader.read(&mut buffer[(*cursor as usize)..]).await?;
                    if read == 0 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "while reading data frame header",
                        )
                        .into());
                    }
                    *cursor += read as u8;
                }
                let header = Header::try_from(*buffer)?;
                self.header = Ok(header);
                self.header.as_mut().expect("set to OK above")
            }
        };

        let MiniProtocolState {
            read_buffer,
            read_state,
            server_send_back: send_back,
            client_send_backs,
        } = &mut state[protocol.index()];
        read_buffer.reserve(*remaining as usize);
        let mut initial_position = read_buffer.len();

        while let read @ 1.. = reader
            .read_buf(&mut read_buffer.limit(*remaining as usize))
            .await?
        {
            *remaining -= read as u16;
        }
        if *remaining != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "while reading payload",
            )
            .into());
        }

        while initial_position != read_buffer.len() {
            let mut decoder = Decoder(&read_buffer[initial_position..]);
            match read_state.feed(&mut decoder) {
                Err(tinycbor::container::Error::Malformed(
                    tinycbor::primitive::Error::EndOfInput,
                )) => break,
                Err(_) => {
                    return Err(MuxError::Malformed(
                        read_buffer
                            .split_to(read_buffer.len() - decoder.0.len())
                            .freeze(),
                    ));
                }
                Ok(()) => {}
            }

            let message = read_buffer
                .split_to(read_buffer.len() - decoder.0.len())
                .freeze();
            initial_position = 0;
            let sender = if *server_sent {
                client_send_backs
                    .front_mut()
                    .ok_or(MuxError::UnexpectedMessage(message.clone()))?
                // TODO: figure out how to know if the server keeps the agency.
                // Probably need to check the message against the state machine to know if the new
                // state has server or client agency.
                // client_send_backs.pop_front();
            } else {
                &mut *send_back
            };
            if let Err(TrySendError::Full(_)) = sender.try_send(Ingress {
                message,
                timestamp: *timestamp,
            }) {
                return Err(MuxError::Full {
                    protocol: *protocol,
                    server: !*server_sent,
                });
            }
        }

        self.header = Err(([0; 8], 0));
        Ok(())
    }
}
