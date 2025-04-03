use std::{convert::Infallible, error::Error, fmt::Display, io, pin::pin};

use futures::{AsyncRead, AsyncWrite, AsyncWriteExt, StreamExt, channel::mpsc::Receiver};
use header::{Header, ProtocolNumber};
use minicbor::{Decode, Encode};

use crate::{
    traits::{
        message::Message,
        mini_protocol::{self, MiniProtocol},
        protocol::{self, Protocol, UnknownProtocol},
        state::{self, Agency, State},
    },
    typefu::{
        Func, FuncOnce, Poly, PolyOnce, ToRef,
        coproduct::{CoproductFoldable, CoproductMappable, Overwrite},
        map::{CMap, HMap, Identity, TypeMap},
    },
};

mod header;

pub struct Mux<P> {
    protocol: std::marker::PhantomData<P>,
}

pub struct Client<MP> {
    mini_protocol: std::marker::PhantomData<MP>,
}

#[derive(Debug)]
pub enum MuxError {
    Io(io::Error),
    Protocol(UnknownProtocol),
    Decode(minicbor::decode::Error),
    Encode(minicbor::encode::Error<Infallible>),
}

impl From<io::Error> for MuxError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<UnknownProtocol> for MuxError {
    fn from(e: UnknownProtocol) -> Self {
        Self::Protocol(e)
    }
}

impl From<minicbor::decode::Error> for MuxError {
    fn from(e: minicbor::decode::Error) -> Self {
        Self::Decode(e)
    }
}

impl From<minicbor::encode::Error<Infallible>> for MuxError {
    fn from(e: minicbor::encode::Error<Infallible>) -> Self {
        Self::Encode(e)
    }
}

impl Display for MuxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Protocol(e) => write!(f, "Unknown protocol: {}", e),
            Self::Decode(e) => write!(f, "Decode error: {}", e),
            Self::Encode(e) => write!(f, "Encode error: {}", e),
        }
    }
}

impl Error for MuxError {}

async fn writer_task<P>(
    mut writer: impl AsyncWrite,
    mut rx: Receiver<protocol::Message<P>>,
) -> Result<Infallible, MuxError>
where
    P: Protocol,
    CMap<mini_protocol::Message>: TypeMap<P>,
    HMap<Identity>: TypeMap<P, Output: Default>,
    // Get ref + 'static
    protocol::Message<P>: for<'a> ToRef<'a> + 'static,
    // Sent from server or client? + Encode<()> + Get protocol number from message
    for<'a> <protocol::Message<P> as ToRef<'a>>::Output: CoproductFoldable<Poly<FromAgency>, Option<bool>>
        + CoproductFoldable<
            PolyOnce<&'a mut minicbor::Encoder<Vec<u8>>>,
            Result<(), minicbor::encode::Error<Infallible>>,
        > + CoproductMappable<Overwrite<protocol::List<P>>, Output = P>,
{
    let mut writer = pin!(writer);
    let encode_buffer = Vec::with_capacity(64 * 1024);
    let mut encoder = minicbor::Encoder::new(encode_buffer);
    let mut peeked: Option<protocol::Message<P>> = None;
    let time = std::time::Instant::now();

    while let Some(message) = {
        if peeked.is_none() {
            rx.next().await
        } else {
            peeked.take()
        }
    } {
        let server_sent = message.to_ref().fold(Poly(FromAgency));
        message.to_ref().fold(PolyOnce(&mut encoder))?;

        let protocol = message
            .to_ref()
            .map(Overwrite(protocol::List::<P>::default()));

        loop {
            if encoder.writer().len() >= u16::MAX as usize {
                break;
            }
            if let Ok(Some(next_message)) = rx.try_next() {
                if next_message
                    .to_ref()
                    .map(Overwrite(protocol::List::<P>::default()))
                    == protocol
                {
                    next_message.to_ref().fold(PolyOnce(&mut encoder))?;
                } else {
                    peeked = Some(next_message);
                    break;
                }
            } else {
                break;
            }
        }

        for chunk in encoder.writer().chunks(u16::MAX as usize) {
            let header = Header {
                timestamp: time.elapsed().as_micros() as u32,
                protocol: ProtocolNumber {
                    protocol,
                    server: server_sent.expect("message is not sent from Done state"),
                },
                payload_len: chunk.len() as u16,
            };
            let header_bytes: [u8; 8] = header.into();

            writer.write_all(&header_bytes).await?;
            writer.write_all(chunk).await?;
        }
    }

    Err(MuxError::Io(std::io::Error::new(
        std::io::ErrorKind::BrokenPipe,
        "All senders have disconnected",
    )))
}

async fn reader_task<P>(reader: impl AsyncRead, senders: protocol::List<P>) {

    // The senders: HList of MiniProtocol message, with the state of the mini protocol.
    struct MessageSender<MP> {
        sender: futures::channel::oneshot::Sender
    }

    
    // we recieve a Protocol Message which can be deconstructed into a mini protocol message
    // because of the protocol number associated with it.
    // This mini protocol message is a coproduct of the message of the different states.
    //
    // The structure:
    // - For every MiniProtocol, we have a receiver that receive the senders through which the
    // messages should be sent.
}

impl<Input> FuncOnce<&Input> for &'_ mut minicbor::Encoder<Vec<u8>>
where
    Input: Encode<()>,
{
    type Output = Result<(), minicbor::encode::Error<Infallible>>;

    fn call(self, input: &Input) -> Self::Output {
        input.encode(self, &mut ())
    }
}

impl<MP> FuncOnce<&MP> for &'_ mut minicbor::Decoder<'_>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP, Output: for<'a> Decode<'a, ()>>,
{
    type Output = Result<<mini_protocol::Message as TypeMap<MP>>::Output, minicbor::decode::Error>;

    fn call(self, _: &MP) -> Self::Output {
        self.decode()
    }
}
