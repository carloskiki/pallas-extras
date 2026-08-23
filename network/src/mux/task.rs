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

                if let Err(e) = writer_task(
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

async fn writer_task(
    writer: &mut (impl AsyncWrite + Unpin),
    message: Egress,
    time: &std::time::Instant,
) -> Result<(), MuxError> {
    let message = message.finalize(Timestamp::elapsed(time));
    writer.write_all(&message).await.map_err(MuxError::Io)
}

pub struct State {
    pub(crate) read_buffer: BytesMut,
    pub(crate) read_position: usize,
    pub(crate) read_state: tinycbor::stream::Any,
    pub(crate) server_send_back: Sender<Ingress>,
    pub(crate) client_send_back: Sender<Ingress>,
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
        while self.remaining != 0 {
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
        let _timestamp = header.timestamp;

        let State {
            read_buffer,
            read_position,
            read_state,
            server_send_back,
            client_send_back,
        } = P::get_state(protocol, state).ok_or(MuxError::UnknownProtocol(protocol))?;
        read_buffer.reserve(remaining.get() as usize);

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

        while *read_position != read_buffer.len() {
            let mut decoder = Decoder(&read_buffer[*read_position..]);
            let feed_result = read_state.feed(&mut decoder);
            let consumed = read_buffer[*read_position..].len() - decoder.0.len();
            *read_position += consumed;
            match feed_result {
                Err(tinycbor::container::Error::Malformed(
                    tinycbor::primitive::Error::EndOfInput,
                )) => break,
                Err(_) => {
                    return Err(MuxError::Malformed(read_buffer.clone().freeze()));
                }
                Ok(()) => {}
            }

            let message = read_buffer.split_to(*read_position).freeze();

            let send_back = if protocol.server_sent() {
                &mut *client_send_back
            } else {
                &mut *server_send_back
            };
            if let Err(TrySendError::Full(_)) = send_back.try_send(Ingress { message }) {
                return Err(MuxError::Full(protocol));
            }

            *read_position = 0;
            read_state.reset();
        }

        self.remaining = 8;
        Ok(())
    }
}
