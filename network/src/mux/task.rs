use crate::{
    Protocol,
    mux::{
        Egress, Ingress, MuxError,
        header::{Header, Timestamp},
    },
};
use bytes::{BufMut, BytesMut};
use tinycbor::Decoder;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    select,
    sync::mpsc::{Receiver, Sender, error::TrySendError},
};

pub(super) async fn task<P>(
    mut bearer: impl AsyncRead + AsyncWrite + Unpin,
    mut receiver: Receiver<Egress>,
    mut state: P::State,
) -> MuxError
where
    P: Protocol,
{
    let time = std::time::Instant::now();
    let mut reader_task = ReadTask {
        header: [0; _],
        remaining: 8,
    };

    loop {
        select! {
            request = receiver.recv() => {
                let Some(request) = request else {
                    return MuxError::Closed;
                };

                if let Err(e) = writer_task::<P>(
                    &mut bearer,
                    request,
                    &time,
                ).await {
                    return e;
                }
            },
            result = reader_task.read_message::<P>(&mut bearer, &mut state) => {
                if let Err(e) = result {
                    return e;
                }
            }
        }
    }
}

async fn writer_task<P: Protocol>(
    writer: &mut (impl AsyncWrite + Unpin),
    message: Egress,
    time: &std::time::Instant,
) -> Result<(), MuxError> {
    let message = message.finalize(Timestamp::elapsed(time));
    writer.write_all(&message).await.map_err(MuxError::Io)
}

pub struct State {
    pub read_buffer: BytesMut,
    pub read_state: tinycbor::stream::Any,
    pub server_send_back: Sender<Ingress>,
    pub client_send_back: Sender<Ingress>,
}

struct ReadTask {
    header: [u8; 8],
    remaining: u8,
}

impl ReadTask {
    /// Read messages from the bearer, and send them to the appropriate handle.
    ///
    /// The future returned by this method is cancel safe.
    async fn read_message<P: Protocol>(
        &mut self,
        reader: &mut (impl AsyncRead + Unpin),
        state: &mut P::State,
    ) -> Result<(), MuxError> {
        if self.remaining != 0 {
            let read = reader
                .read(&mut self.header[8 - self.remaining as usize..])
                .await?;
            if read == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "while reading data frame header",
                )
                .into());
            }
            self.remaining -= read as u8;
        }
        let header: &mut Header = zerocopy::transmute_mut!(&mut self.header);
        let remaining = &mut header.payload_len;
        let protocol = header.protocol;
        let timestamp = header.timestamp;

        let State {
            read_buffer,
            read_state,
            server_send_back,
            client_send_back,
        } = P::get_state(protocol, state).ok_or(MuxError::UnknownProtocol(protocol))?;
        read_buffer.reserve(remaining.get() as usize);
        let mut initial_position = read_buffer.len();

        while let read @ 1.. = reader
            .read_buf(&mut read_buffer.limit(remaining.get() as usize))
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
            let feed_result = read_state.feed(&mut decoder);
            let message = read_buffer
                .split_to(read_buffer.len() - decoder.0.len())
                .freeze();
            match feed_result {
                Err(tinycbor::container::Error::Malformed(
                    tinycbor::primitive::Error::EndOfInput,
                )) => break,
                Err(_) => {
                    return Err(MuxError::Malformed(message));
                }
                Ok(()) => {}
            }

            let send_back = if protocol.server_sent() {
                &mut *server_send_back
            } else {
                &mut *client_send_back
            };
            if let Err(TrySendError::Full(_)) = send_back.try_send(Ingress { message, timestamp }) {
                return Err(MuxError::Full(protocol));
            }

            initial_position = 0;
            read_state.reset();
        }

        self.remaining = 8;
        Ok(())
    }
}
