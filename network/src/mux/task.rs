use super::{
    MuxError, TaskState,
    header::{Header, ProtocolNumber},
};
use crate::{mux::Request, traits::protocol::Protocol, typefu::array::Index};
use bytes::{Bytes, BytesMut};
use hybrid_array::Array;
use std::{
    collections::VecDeque,
    convert::Infallible,
    ops::DerefMut,
    pin::{Pin, pin},
    sync::mpsc::SendError,
    task::{Poll, ready},
};
use tinycbor::Decoder;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    select,
    sync::mpsc::{Receiver, Sender, error::TrySendError},
};
use tokio_stream::StreamExt;

#[allow(private_bounds)]
pub(super) async fn task<P>(
    mut bearer: impl AsyncRead + AsyncWrite,
    mut receiver: Receiver<Request<P>>,
    mut state: Array<TaskState, P::Length>,
) -> Result<Infallible, MuxError<P>>
where
    P: Protocol + Index,
{
    let mut bearer = pin!(bearer);
    let mut decode_buffer = Vec::with_capacity(64 * 1024);
    let mut previous_state: Option<(P, usize)> = None;
    let time = std::time::Instant::now();
    let mut reader_task = ReaderTask {
        header: Err(([0; _], 0)),
        state: &mut state,
    };

    loop {
        select! {
            request = receiver.recv() => {
                writer_task(
                    &mut bearer,
                    &mut receiver,
                    &time,
                    &mut state,
                ).await?;
            },
            result = reader_task.read_message(bearer) => {
                result?;
            }
        }
    }
}

async fn writer_task<'b, P>(
    mut writer: Pin<&mut impl AsyncWrite>,
    Request {
        message,
        protocol,
        send_back,
    }: Request<P>,
    time: &std::time::Instant,
    state: &mut Array<TaskState, P::Length>,
) -> Result<(), MuxError<P>>
where
    P: Protocol + Index,
{
    // Write the timestamp to the header. TODO: this should be a wrapper struct over the buffer to
    // better document the behavior and ensure correctness.
    message.copy_from_slice(&(time.elapsed().as_micros() as u32).to_be_bytes());
    // 1. Send the bytes
    writer.write_all(&message).await?;
    // TODO: buffer pool.
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
    server_send_back: Sender<(Bytes, std::time::Duration)>,
    client_send_backs: VecDeque<Sender<(Bytes, std::time::Duration)>>,
}

struct ReaderTask<'a, P: Protocol + Index> {
    header: Result<Header<P>, ([u8; 8], u8)>,
    state: &'a mut Array<MiniProtocolState, P::Length>,
}

impl<P: Protocol + Index> ReaderTask<'_, P> {
    /// Read a message from the reader and return it.
    ///
    /// The future returned by this method is cancel safe.
    async fn read_message(&mut self, mut reader: impl AsyncRead + Unpin) -> Result<(), MuxError> {
        let Header {
            timestamp,
            protocol:
                ProtocolNumber {
                    protocol,
                    server_sent,
                },
            payload_len: remaining,
        } = match self.header {
            Ok(header) => header,
            Err((buffer, cursor)) => {
                while cursor != 8 {
                    let read = reader.read(&mut buffer[(cursor as usize)..]).await?;
                    if read == 0 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "while reading data frame header",
                        )
                        .into());
                    }
                    cursor += read as u8;
                }
                let header = Header::try_from(buffer)?;
                self.header = Ok(header);
                header
            }
        };

        let MiniProtocolState {
            read_buffer,
            read_state,
            server_send_back: send_back,
            client_send_backs,
        } = &mut self.state[protocol.index()];
        read_buffer.reserve(remaining as usize);
        let mut initial_position = read_buffer.len();

        let mut limited = bytes::BufMut::limit(read_buffer, remaining as usize);
        while reader.read_buf(&mut limited).await? {}
        if limited.limit() != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected EOF while reading payload",
            )
            .into());
        }

        while initial_position != read_buffer.len() {
            let mut decoder = Decoder(&read_buffer[initial_position..]);
            match read_state.feed(&mut decoder) {
                Err(tinycbor::container::Error::Malformed(
                    tinycbor::primitive::Error::EndOfInput,
                )) => break,
                Err(e) => return Err(MuxError::Malformed),
                Ok(()) => {}
            }

            let remaining = decoder.0.len();
            let message = read_buffer.split_to(read_buffer.len() - remaining).freeze();
            initial_position = 0;
            let sender = if server_sent {
                client_send_backs
                    .front_mut()
                    .ok_or(MuxError::UnexpectedMessage(
                        Vec::from(message).into_boxed_slice(),
                    ))?
                // TODO: figure out how to know if the server keeps the agency.
                // Probably need to check the message against the state machine to know if the new
                // state has server or client agency.
                // client_send_backs.pop_front();
            } else {
                send_back
            };
            // TODO: timestamp management in header struct directly.
            let timestamp = std::time::Duration::from_micros(timestamp as u64);
            if let Err(TrySendError::Full(_)) = sender.try_send((message, timestamp)) {
                return Err(MuxError::Full {
                    protocol,
                    server: !server_sent,
                });
            }
        }

        Ok(())
    }
}
