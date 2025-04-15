// Writer Task
//
// Receiver: Message & sendback pair for <P>
// Getting the agency of the message:
// - Message<MP> -> Agency
//
// When do we need agency?
//
// For the client & server to implement the right function OK
// For the writer to send a message with the correct tag OK - Every message has a valid from state
// For the reader know whether to remove the client receiver from the queue OK - if we make the Done
// state a client agency.

use std::{
    collections::VecDeque,
    convert::Infallible,
    ops::DerefMut,
    pin::pin,
    task::{Poll, ready},
};

use futures::{
    AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, FutureExt, SinkExt, StreamExt,
    channel::mpsc::{Receiver, Sender, UnboundedSender},
    sink::Feed,
};
use minicbor::{Decode, Encode, decode_with};

use crate::{
    traits::{
        mini_protocol::{self, DecodeContext, EncodeContext, MiniProtocol},
        protocol::{self, Protocol},
        state::{self, Agency, State},
    },
    typefu::{
        Func, FuncOnce, Poly, PolyOnce, ToMut, ToRef,
        coproduct::{CoproductFoldable, CoproductMappable, Overwrite},
        map::{CMap, HMap, Identity, TypeMap, Zip},
    },
};

use super::{
    MiniProtocolSendBundle, MiniProtocolTaskState, MuxError, ProtocolSendBundle, ProtocolTaskState,
    SendBundle, TaskState,
    header::{Header, ProtocolNumber},
};

pub(super) async fn task<P>(
    mut bearer: impl AsyncRead + AsyncWrite,
    mut rx: Receiver<ProtocolSendBundle<P>>,
    mut task_state: ProtocolTaskState<P>,
) -> Result<Infallible, MuxError>
where
    P: Protocol,
    HMap<Identity>: TypeMap<P, Output: Default>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
    HMap<MiniProtocolTaskState>: TypeMap<P>,
{
    todo!()
}

async fn writer_task<P>(
    mut writer: impl AsyncWrite,
    mut rx: Receiver<ProtocolSendBundle<P>>,
    task_state: &mut ProtocolTaskState<P>,
) -> Result<Infallible, MuxError>
where
    P: Protocol,
    HMap<Identity>: TypeMap<P, Output: Default>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
    HMap<MiniProtocolTaskState>: TypeMap<P>,
    // Get a ref to the send bundle
    ProtocolSendBundle<P>: for<'a> ToRef<'a>,
    // Get Protocol of send bundle + Get the agency of the sender
    for<'a> BundleRef<'a, P>: CoproductMappable<Overwrite<protocol::List<P>>, Output = P>
        + CoproductFoldable<Poly<MessageFromAgency>, bool>,
    // Get a mutable reference to the mini protocol state
    ProtocolTaskState<P>: for<'a> ToMut<'a>,
    // Zip the message and state
    for<'a> Zip<<ProtocolTaskState<P> as ToMut<'a>>::Output>: FuncOnce<ProtocolSendBundle<P>>,
    // Process the bundle:
    // - Encode the message in the buffer
    // - Add the send_back in the queue if sent from client
    for<'a, 'b> CMap<ProcessBundle<'a>>:
        FuncOnce<Zipped<'b, P>, Output = Result<(), minicbor::encode::Error<Infallible>>>,
{
    // Plan with one bundle
    // - Get the protocol for the message
    // - WHILE the buffer is not full:
    //      - Get the next message if it is ready else send the message & break.
    //      - if the next message is not the same protocol as this one, put it in peeked and send
    //      the message & break.
    //      - Encode the the message.
    //      - Check if the message is sent from server & add the send_back in the queue

    let mut writer = pin!(writer);
    let encode_buffer = Vec::with_capacity(64 * 1024);
    let mut encoder = minicbor::Encoder::new(encode_buffer);
    let mut peeked = None;
    let time = std::time::Instant::now();

    while let Some(send_bundle) = {
        if peeked.is_none() {
            rx.next().await
        } else {
            peeked.take()
        }
    } {
        let protocol = send_bundle
            .to_ref()
            .map(Overwrite(protocol::List::<P>::default()));
        let server_sent = send_bundle.to_ref().fold(Poly(MessageFromAgency));

        CMap(ProcessBundle {
            encoder: &mut encoder,
        })
        .call_once(Zip(task_state.to_mut()).call_once(send_bundle))?;
        loop {
            if encoder.writer().len() >= u16::MAX as usize {
                break;
            }
            if let Ok(Some(send_bundle)) = rx.try_next() {
                if send_bundle
                    .to_ref()
                    .map(Overwrite(protocol::List::<P>::default()))
                    == protocol
                    && send_bundle.to_ref().fold(Poly(MessageFromAgency)) == server_sent
                {
                    CMap(ProcessBundle {
                        encoder: &mut encoder,
                    })
                    .call_once(Zip(task_state.to_mut()).call_once(send_bundle))?;
                } else {
                    peeked = Some(send_bundle);
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
                    server_sent,
                },
                payload_len: chunk.len() as u16,
            };
            let header_bytes: [u8; 8] = header.into();

            writer.write_all(&header_bytes).await?;
            writer.write_all(chunk).await?;
        }
    }

    Err(MuxError::HandleDropped)
}

type BundleRef<'a, P> = <ProtocolSendBundle<P> as ToRef<'a>>::Output;
type Zipped<'a, P> =
    <Zip<<ProtocolTaskState<P> as ToMut<'a>>::Output> as TypeMap<ProtocolSendBundle<P>>>::Output;

/// Getting the agency of the sender
struct MessageFromAgency;
impl<MP> TypeMap<&SendBundle<MP>> for MessageFromAgency
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    type Output = bool;
}
impl<MP> Func<&SendBundle<MP>> for MessageFromAgency
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
    for<'a> &'a mini_protocol::Message<MP>: CoproductMappable<Overwrite<MP::States>, Output: CoproductFoldable<Poly<StateAgency>, bool>>,
{
    #[inline]
    fn call(SendBundle { message, .. }: &SendBundle<MP>) -> Self::Output {
        message
            .map(Overwrite(MP::States::default()))
            .fold(Poly(StateAgency))
    }
}
struct StateAgency;
impl<S> TypeMap<S> for StateAgency
where
    S: State,
{
    type Output = bool;
}
impl<S> Func<S> for StateAgency
where
    S: State,
{
    #[inline]
    fn call(_: S) -> Self::Output {
        S::Agency::SERVER
    }
}

impl<Input> TypeMap<&Input> for &'_ mut minicbor::Encoder<Vec<u8>>
where
    Input: Encode<EncodeContext>,
{
    type Output = Result<(), minicbor::encode::Error<Infallible>>;
}

impl<Input> FuncOnce<&Input> for &'_ mut minicbor::Encoder<Vec<u8>>
where
    Input: Encode<EncodeContext>,
{
    #[inline]
    fn call_once(self, input: &Input) -> Self::Output {
        input.encode(self, &mut EncodeContext)
    }
}

/// Process the bundle and return whether the bundle was sent by a server.
type Bundle<'a, MP> = (&'a mut TaskState<MP>, SendBundle<MP>);
struct ProcessBundle<'a> {
    encoder: &'a mut minicbor::Encoder<Vec<u8>>,
}
impl<MP> TypeMap<Bundle<'_, MP>> for ProcessBundle<'_>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    type Output = Result<(), minicbor::encode::Error<Infallible>>;
}
impl<MP> FuncOnce<Bundle<'_, MP>> for ProcessBundle<'_>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
    mini_protocol::Message<MP>: Encode<EncodeContext>,
{
    #[inline]
    fn call_once(self, (task_state, mut send_bundle): Bundle<'_, MP>) -> Self::Output {
        self.encoder.call_once(&send_bundle.message)?;
        if let Some(send_back) = send_bundle.send_back.take() {
            task_state.client_send_backs.push_back(send_back);
        }
        Ok(())
    }
}

async fn reader_task<P>(
    reader: impl AsyncRead,
    task_state: &mut ProtocolTaskState<P>,
) -> Result<Infallible, MuxError>
where
    P: Protocol,
    HMap<MiniProtocolTaskState>: TypeMap<P>,
    ProtocolTaskState<P>: for<'a> ToMut<'a>,
    for<'a> Zip<<ProtocolTaskState<P> as ToMut<'a>>::Output>: FuncOnce<P>,
    for<'a, 'b> CMap<ProcessMessage<'a, 'b>>:
        FuncOnce<ReaderZipped<'a, P>, Output: Future<Output = Result<(), MuxError>>>,
{
    let mut reader = pin!(reader);
    let mut buffer: Vec<u8> = Vec::with_capacity(64 * 1024);
    let mut previous_state: Option<(P, usize)> = None;
    loop {
        let mut header_buffer = [0; 8];
        reader.read_exact(&mut header_buffer).await?;
        let Header {
            protocol:
                ProtocolNumber {
                    protocol,
                    server_sent,
                },
            payload_len,
            ..
        } = Header::<P>::try_from(header_buffer)?;
        if previous_state.is_some_and(|(p, _)| p != protocol) {
            return Err(MuxError::InvalidPeerMessage);
        }

        let read_len = reader
            .as_mut()
            .take(payload_len as u64)
            .read_to_end(&mut buffer)
            .await?;
        if read_len != payload_len as usize {
            return Err(MuxError::InvalidPeerMessage);
        }
        let mut decoder = minicbor::decode::Decoder::new(&buffer);
        if let Some((_, start_pos)) = previous_state.take() {
            decoder.set_position(start_pos);
        }
        while decoder.position() != decoder.input().len() {
            let result = CMap(ProcessMessage {
                decoder: &mut decoder,
                server_sent,
            })
            .call_once(Zip(task_state.to_mut()).call_once(protocol))
            .await;
            match result {
                Ok(()) => {}
                Err(MuxError::Decode(e)) if e.is_end_of_input() => {
                    previous_state = Some((protocol, decoder.position()));
                    break;
                }
                Err(e) => return Err(e),
            }
        }
        // No bytes left over.
        if previous_state.is_none() {
            buffer.clear();
        }
    }
}
type ReaderZipped<'a, P> = <Zip<<ProtocolTaskState<P> as ToMut<'a>>::Output> as TypeMap<P>>::Output;
type ProcessResult<'a, 'b, P> =
    <CMap<ProcessMessage<'a, 'b>> as TypeMap<ReaderZipped<'a, P>>>::Output;

enum FeedResult<'a, MP>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    ToServer(Feed<'a, Sender<mini_protocol::Message<MP>>, mini_protocol::Message<MP>>),
    ToClientPopped(
        UnboundedSender<mini_protocol::Message<MP>>,
        Option<mini_protocol::Message<MP>>,
    ),
    ToClientKeep(Feed<'a, UnboundedSender<mini_protocol::Message<MP>>, mini_protocol::Message<MP>>),
    Error(MuxError),
}

impl<MP> Unpin for FeedResult<'_, MP>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
}

impl<MP> Future for FeedResult<'_, MP>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    type Output = Result<(), MuxError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match self.deref_mut() {
            FeedResult::ToServer(feed) => feed.poll_unpin(cx).map_err(|_| MuxError::HandleDropped),
            FeedResult::ToClientKeep(feed) => {
                feed.poll_unpin(cx).map_err(|_| MuxError::HandleDropped)
            }
            FeedResult::ToClientPopped(sender, message) => {
                ready!(sender.poll_ready(cx)).map_err(|_| MuxError::HandleDropped)?;
                let item = message.take().expect("polled Feed after completion");
                sender
                    .start_send(item)
                    .map_err(|_| MuxError::HandleDropped)?;
                Poll::Ready(Ok(()))
            }
            FeedResult::Error(err) => Poll::Ready(Err(std::mem::replace(
                err,
                // This is a placeholder, we should never reach here because the future should not
                // be polled after it completes.
                MuxError::HandleDropped,
            ))),
        }
    }
}

type ReceiverState<'a, MP> = (&'a mut TaskState<MP>, MP);
struct ProcessMessage<'a, 'b> {
    decoder: &'a mut minicbor::Decoder<'b>,
    server_sent: bool,
}
impl<'a, MP> TypeMap<ReceiverState<'a, MP>> for ProcessMessage<'_, '_>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States, Output: for<'b> Decode<'b, DecodeContext>>,
{
    type Output = FeedResult<'a, MP>;
}
impl<'a, MP> FuncOnce<ReceiverState<'a, MP>> for ProcessMessage<'_, '_>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States, Output: for<'b> Decode<'b, DecodeContext>>,
    MessageToAgency<MP>: for<'b> Func<&'b mini_protocol::Message<MP>, Output = bool>,
{
    #[inline]
    fn call_once(self, (task_state, _): ReceiverState<'a, MP>) -> Self::Output {
        let message = match self.decoder.decode_with(&mut DecodeContext(None)) {
            Ok(m) => m,
            Err(e) => return FeedResult::Error(MuxError::Decode(e)),
        };
        let server_has_agency_next = MessageToAgency::<MP>::call(&message);
        if self.server_sent {
            if server_has_agency_next {
                let Some(sender) = task_state.client_send_backs.front_mut() else {
                    return FeedResult::Error(MuxError::InvalidPeerMessage);
                };
                FeedResult::ToClientKeep(sender.feed(message))
            } else {
                let Some(sender) = task_state.client_send_backs.pop_front() else {
                    return FeedResult::Error(MuxError::InvalidPeerMessage);
                };
                FeedResult::ToClientPopped(sender, Some(message))
            }
        } else {
            FeedResult::ToServer(task_state.server_send_back.feed(message))
        }
    }
}

pub struct MessageToAgency<MP>(MP);

impl<MP> TypeMap<&mini_protocol::Message<MP>> for MessageToAgency<MP>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    type Output = bool;
}
impl<'a, MP> Func<&'a mini_protocol::Message<MP>> for MessageToAgency<MP>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
    &'a mini_protocol::Message<MP>: CoproductMappable<Overwrite<MP::States>, Output: CoproductFoldable<Poly<StateAgency>, bool>>,
{
    fn call(i: &'a mini_protocol::Message<MP>) -> Self::Output {
        i.map(Overwrite(MP::States::default()))
            .fold(Poly(StateAgency))
    }
}
