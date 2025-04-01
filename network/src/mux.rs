use std::{convert::Infallible, error::Error, fmt::Display, io, pin::pin};

use frunk::{
    Coproduct, Func, HCons, Poly, ToRef,
    coproduct::{CoproductFoldable, CoproductMappable},
};
use futures::{AsyncRead, AsyncWrite, AsyncWriteExt, StreamExt, channel::mpsc::Receiver};
use header::{Header, ProtocolNumber};
use minicbor::{Decode, Encode};

use crate::{
    protocol::{Agency, Message, MiniProtocol, Protocol, State, UnknownProtocol},
    typefu::{
        Constructor, FuncOnce, HMap, MiniProtocolMessage, PolyOnce, ProtocolList, ProtocolMessage,
        TypeMap, constructors,
        type_maps::{self, Identity},
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
    mut rx: Receiver<ProtocolMessage<P>>,
) -> Result<Infallible, MuxError>
where
    P: Protocol,
    ProtocolList<P>: Constructor<HMap<DefaultFn>>,
    // Get ref + 'static
    ProtocolMessage<P>: for<'a> ToRef<'a> + 'static,
    // Sent from server or client? + Encode<()> + Get protocol number from message
    for<'a> <ProtocolMessage<P> as ToRef<'a>>::Output: CoproductFoldable<Poly<FromAgency>, bool>
        + CoproductFoldable<
            PolyOnce<&'a mut minicbor::Encoder<Vec<u8>>>,
            Result<(), minicbor::encode::Error<Infallible>>,
        > + CoproductMappable<ProtocolConstructor<P>, Output = P>,
{
    let mut writer = pin!(writer);
    let encode_buffer = Vec::with_capacity(64 * 1024);
    let mut encoder = minicbor::Encoder::new(encode_buffer);
    let mut peeked: Option<ProtocolMessage<P>> = None;
    let time = std::time::Instant::now();

    while let Some(message) = {
        if peeked.is_none() {
            rx.next().await
        } else {
            peeked.take()
        }
    } {
        let server_sent = message.to_ref().fold(Poly(FromAgency));
        message.to_ref().fold(PolyOnce(&mut encoder));

        let protocol = message.to_ref().map(ProtocolList::<P>::construct());

        loop {
            if encoder.writer().len() >= u16::MAX as usize {
                break;
            }
            if let Ok(Some(next_message)) = rx.try_next() {
                if next_message.to_ref().map(ProtocolList::<P>::construct()) == protocol {
                    next_message.to_ref().fold(PolyOnce(&mut encoder));
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
                    server: server_sent,
                },
                payload_len: chunk.len() as u16,
            };
            let header_bytes: [u8; 8] = header.into();

            writer.write_all(&header_bytes).await?;
            writer.write_all(chunk).await?;
        }
    }

    return Err(MuxError::Io(std::io::Error::new(
        std::io::ErrorKind::BrokenPipe,
        "All senders have disconnected",
    )));
}

async fn reader_task<P>(reader: impl AsyncRead, senders: P) {
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

struct FromAgency;

impl<T: Message> Func<T> for FromAgency {
    type Output = bool;

    fn call(_: T) -> Self::Output {
        <T::FromState as State>::Agency::SERVER
    }
}

impl<MP> FuncOnce<&MP> for &'_ mut minicbor::Decoder<'_>
where
    MP: MiniProtocol,
    MiniProtocolMessage<MP>: for<'a> Decode<'a, ()>,
{
    type Output = Result<MiniProtocolMessage<MP>, minicbor::decode::Error>;

    fn call(self, _: &MP) -> Self::Output {
        self.decode()
    }
}

pub struct DefaultFn;

impl<Input> TypeMap<Input> for DefaultFn {
    type Output = fn() -> Input;
}

impl<Input: Default> Constructor<Input> for DefaultFn {
    fn construct() -> Self::Output {
        Default::default
    }
}

type ProtocolConstructor<P> =
    <<P as TypeMap<HMap<Identity>>>::Output as TypeMap<HMap<DefaultFn>>>::Output;
