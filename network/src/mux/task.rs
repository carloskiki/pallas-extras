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
    convert::Infallible,
    marker::PhantomData,
    ops::DerefMut,
    pin::{Pin, pin},
    task::{Poll, ready},
};

use futures::{
    AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, FutureExt, SinkExt, StreamExt,
    channel::mpsc::{Receiver, Sender, UnboundedSender},
    future::FusedFuture,
    select,
    sink::Feed,
};
use minicbor::{Decode, Encode};

use crate::{
    traits::{
        message::Message,
        mini_protocol::{self, DecodeContext, EncodeContext, MiniProtocol},
        protocol::{self, Protocol},
        state::{self, Agency, State},
    },
    typefu::{
        Func, FuncOnce, ToMut, ToRef,
        coproduct::CNil,
        map::{CMap, Fold, HMap, Identity, Overwrite, TypeMap, Zip},
    },
};

use super::{
    MiniProtocolSendBundle, MiniProtocolTaskState, MuxError, ProtocolSendBundle, ProtocolTaskState,
    SendBundle, TaskState,
    header::{Header, ProtocolNumber},
};

#[allow(private_bounds)]
pub(super) async fn task<P>(
    mut bearer: impl AsyncRead + AsyncWrite,
    mut receiver: Receiver<ProtocolSendBundle<P>>,
    mut task_state: ProtocolTaskState<P>,
) -> Result<Infallible, MuxError>
where
    P: Protocol,
    HMap<Identity>: TypeMap<P, Output: Default>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
    HMap<MiniProtocolTaskState>: TypeMap<P>,
    // Get a ref to the send bundle
    ProtocolSendBundle<P>: for<'a> ToRef<'a>,
    // Get Protocol of send bundle + Get the agency of the sender
    Overwrite<protocol::List<P>>: for<'a> FuncOnce<BundleRef<'a, P>, Output = P>,
    Fold<MessageFromAgency, bool>: for<'a> Func<BundleRef<'a, P>, Output = bool>,
    // Get a mutable reference to the mini protocol state
    ProtocolTaskState<P>: for<'a> ToMut<'a>,
    // Zip the message and state
    for<'a> Zip<<ProtocolTaskState<P> as ToMut<'a>>::Output>: FuncOnce<ProtocolSendBundle<P>>,
    // Process the bundle:
    // - Encode the message in the buffer
    // - Add the send_back in the queue if sent from client
    for<'a, 'b, 'c> Fold<ProcessBundle<'a, 'b>, Result<(), minicbor::encode::Error<Infallible>>>:
        FuncOnce<WriterZipped<'c, P>, Output = Result<(), minicbor::encode::Error<Infallible>>>,
    // Get the task state for the protocol of the message received
    for<'a> Zip<<ProtocolTaskState<P> as ToMut<'a>>::Output>: FuncOnce<P>,
    // Decode and send the message to the correct handle.
    for<'a, 'b> CMap<ProcessMessage<'a, 'b>>:
        FuncOnce<ReaderZipped<'a, P>, Output: Future<Output = Result<(), MuxError>> + Send>,
{
    let mut bearer = pin!(bearer);
    let mut encode_buffer = Vec::with_capacity(64 * 1024);
    let mut decode_buffer = Vec::with_capacity(64 * 1024);
    let mut peeked = None;
    let mut previous_state: Option<(P, usize)> = None;
    let time = std::time::Instant::now();
    let mut header_buffer = [0; 8];

    loop {
        select! {
            next_message = NextMessage { receiver: &mut receiver, peeked: &mut peeked } => {
                writer_task(
                    &mut bearer,
                    &mut receiver,
                    &mut task_state,
                    &mut peeked,
                    &time,
                    &mut encode_buffer,
                    next_message?,
                ).await?;
            },
            result = bearer.read_exact(&mut header_buffer).fuse() => {
                result?;
                let header = Header::<P>::try_from(header_buffer)?;
                reader_task(
                    &mut bearer,
                    &mut task_state,
                    &mut previous_state,
                    &mut decode_buffer,
                    header,
                ).await?;
            }
        }
    }
}

struct NextMessage<'a, P>
where
    P: Protocol,
    HMap<Identity>: TypeMap<P, Output: Default>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    receiver: &'a mut Receiver<ProtocolSendBundle<P>>,
    peeked: &'a mut Option<ProtocolSendBundle<P>>,
}

impl<P> Unpin for NextMessage<'_, P>
where
    P: Protocol,
    HMap<Identity>: TypeMap<P, Output: Default>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
}

impl<P> Future for NextMessage<'_, P>
where
    P: Protocol,
    HMap<Identity>: TypeMap<P, Output: Default>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    type Output = Result<ProtocolSendBundle<P>, MuxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if let Some(peeked) = this.peeked.take() {
            Poll::Ready(Ok(peeked))
        } else {
            match this.receiver.poll_next_unpin(cx) {
                Poll::Ready(Some(item)) => Poll::Ready(Ok(item)),
                Poll::Ready(None) => Poll::Ready(Err(MuxError::HandleDropped)),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

impl<P> FusedFuture for NextMessage<'_, P>
where
    P: Protocol,
    HMap<Identity>: TypeMap<P, Output: Default>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    fn is_terminated(&self) -> bool {
        self.peeked.is_none() && futures::stream::FusedStream::is_terminated(&self.receiver)
    }
}

async fn writer_task<'b, P>(
    writer: &mut Pin<&mut impl AsyncWrite>,
    rx: &mut Receiver<ProtocolSendBundle<P>>,
    task_state: &mut ProtocolTaskState<P>,
    peeked: &mut Option<ProtocolSendBundle<P>>,
    time: &std::time::Instant,
    encode_buffer: &'b mut Vec<u8>,
    message: ProtocolSendBundle<P>,
) -> Result<(), MuxError>
where
    P: Protocol,
    HMap<Identity>: TypeMap<P, Output: Default>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
    HMap<MiniProtocolTaskState>: TypeMap<P>,
    // Get a ref to the send bundle
    ProtocolSendBundle<P>: for<'a> ToRef<'a>,
    // Get Protocol of send bundle + Get the agency of the sender
    Overwrite<protocol::List<P>>: for<'a> FuncOnce<BundleRef<'a, P>, Output = P>,
    Fold<MessageFromAgency, bool>: for<'a> Func<BundleRef<'a, P>, Output = bool>,
    // Get a mutable reference to the mini protocol state
    ProtocolTaskState<P>: for<'a> ToMut<'a>,
    // Zip the message and state
    for<'a> Zip<<ProtocolTaskState<P> as ToMut<'a>>::Output>: FuncOnce<ProtocolSendBundle<P>>,
    // Process the bundle:
    // - Encode the message in the buffer
    // - Add the send_back in the queue if sent from client
    for<'a, 'c> Fold<ProcessBundle<'a, 'b>, Result<(), minicbor::encode::Error<Infallible>>>:
        FuncOnce<WriterZipped<'c, P>, Output = Result<(), minicbor::encode::Error<Infallible>>>,
{
    // Plan with one bundle
    // - Get the protocol for the message
    // - WHILE the buffer is not full:
    //      - Get the next message if it is ready else send the message & break.
    //      - if the next message is not the same protocol as this one, put it in peeked and send
    //      the message & break.
    //      - Encode the the message.
    //      - Check if the message is sent from server & add the send_back in the queue

    let mut encoder = minicbor::Encoder::new(encode_buffer);

    let protocol = Overwrite(protocol::List::<P>::default()).call_once(message.to_ref());
    let server_sent = Fold::call(message.to_ref());

    Fold(
        ProcessBundle {
            encoder: &mut encoder,
        },
        PhantomData,
    )
    .call_once(Zip(task_state.to_mut()).call_once(message))?;
    loop {
        if encoder.writer().len() >= u16::MAX as usize {
            break;
        }
        if let Ok(Some(send_bundle)) = rx.try_next() {
            if Overwrite(protocol::List::<P>::default()).call_once(send_bundle.to_ref()) == protocol
                && Fold::call(send_bundle.to_ref()) == server_sent
            {
                Fold(
                    ProcessBundle {
                        encoder: &mut encoder,
                    },
                    PhantomData,
                )
                .call_once(Zip(task_state.to_mut()).call_once(send_bundle))?;
            } else {
                *peeked = Some(send_bundle);
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
    encoder.writer_mut().clear();

    Ok(())
}

pub(super) type BundleRef<'a, P> = <ProtocolSendBundle<P> as ToRef<'a>>::Output;
pub(super) type WriterZipped<'a, P> =
    <Zip<<ProtocolTaskState<P> as ToMut<'a>>::Output> as TypeMap<ProtocolSendBundle<P>>>::Output;

/// Getting the agency of the sender
pub(super) struct MessageFromAgency;
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
    for<'a> Overwrite<MP::States>: FuncOnce<&'a mini_protocol::Message<MP>>,
    Fold<StateAgency, bool>: for<'a> FuncOnce<
            <Overwrite<MP::States> as TypeMap<&'a mini_protocol::Message<MP>>>::Output,
            Output = bool,
        >,
{
    #[inline]
    fn call(SendBundle { message, .. }: &SendBundle<MP>) -> Self::Output {
        Fold(StateAgency, PhantomData)
            .call_once(Overwrite(MP::States::default()).call_once(message))
    }
}
struct StateAgency;
impl<S> TypeMap<S> for StateAgency
where
    S: State,
{
    type Output = bool;
}
impl<S> FuncOnce<S> for StateAgency
where
    S: State,
{
    #[inline]
    fn call_once(self, _: S) -> Self::Output {
        S::Agency::SERVER
    }
}

impl<Input> TypeMap<&Input> for &'_ mut minicbor::Encoder<&'_ mut Vec<u8>>
where
    Input: Encode<EncodeContext>,
{
    type Output = Result<(), minicbor::encode::Error<Infallible>>;
}

impl<Input> FuncOnce<&Input> for &'_ mut minicbor::Encoder<&'_ mut Vec<u8>>
where
    Input: Encode<EncodeContext>,
{
    #[inline]
    fn call_once(self, input: &Input) -> Self::Output {
        input.encode(self, &mut EncodeContext)
    }
}

/// Process the bundle and return whether the bundle was sent by a server.
type Bundle<'a, MP> = (SendBundle<MP>, &'a mut TaskState<MP>);
pub(super) struct ProcessBundle<'a, 'b> {
    encoder: &'a mut minicbor::Encoder<&'b mut Vec<u8>>,
}
impl<MP> TypeMap<Bundle<'_, MP>> for ProcessBundle<'_, '_>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    type Output = Result<(), minicbor::encode::Error<Infallible>>;
}
impl<MP> FuncOnce<Bundle<'_, MP>> for ProcessBundle<'_, '_>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
    mini_protocol::Message<MP>: Encode<EncodeContext>,
{
    #[inline]
    fn call_once(self, (mut send_bundle, task_state): Bundle<'_, MP>) -> Self::Output {
        self.encoder.call_once(&send_bundle.message)?;
        if let Some(send_back) = send_bundle.send_back.take() {
            task_state.client_send_backs.push_back(send_back);
        }
        Ok(())
    }
}
impl TypeMap<CNil> for ProcessBundle<'_, '_> {
    type Output = Result<(), minicbor::encode::Error<Infallible>>;
}
impl FuncOnce<CNil> for ProcessBundle<'_, '_> {
    #[inline]
    fn call_once(self, x: CNil) -> Self::Output {
        match x {}
    }
}

async fn reader_task<P>(
    reader: &mut Pin<&mut impl AsyncRead>,
    task_state: &mut ProtocolTaskState<P>,
    previous_state: &mut Option<(P, usize)>,
    buffer: &mut Vec<u8>,
    Header {
        protocol: ProtocolNumber {
            protocol,
            server_sent,
        },
        payload_len,
        ..
    }: Header<P>,
) -> Result<(), MuxError>
where
    P: Protocol,
    HMap<MiniProtocolTaskState>: TypeMap<P>,
    ProtocolTaskState<P>: for<'a> ToMut<'a>,
    for<'a> Zip<<ProtocolTaskState<P> as ToMut<'a>>::Output>: FuncOnce<P>,
    for<'a, 'b> CMap<ProcessMessage<'a, 'b>>:
        FuncOnce<ReaderZipped<'a, P>, Output: Future<Output = Result<(), MuxError>>>,
{
    if previous_state.is_some_and(|(p, _)| p != protocol) {
        return Err(MuxError::InvalidPeerMessage);
    }

    let read_len = reader
        .as_mut()
        .take(payload_len as u64)
        .read_to_end(buffer)
        .await?;
    if read_len != payload_len as usize {
        return Err(MuxError::InvalidPeerMessage);
    }
    let mut decoder = minicbor::decode::Decoder::new(buffer);
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
                *previous_state = Some((protocol, decoder.position()));
                break;
            }
            Err(e) => return Err(e),
        }
    }

    // No bytes left over.
    if previous_state.is_none() {
        buffer.clear();
    }

    Ok(())
}
pub(super) type ReaderZipped<'a, P> =
    <Zip<<ProtocolTaskState<P> as ToMut<'a>>::Output> as TypeMap<P>>::Output;
pub(super) enum FeedResult<'a, MP>
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

// Hack, we should find a way to not have this.
impl Future for CNil {
    type Output = Result<(), crate::mux::MuxError>;

    fn poll(self: Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        match *self {}
    }
}

type ReceiverState<'a, MP> = (MP, &'a mut TaskState<MP>);
pub(super) struct ProcessMessage<'a, 'b> {
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
    MiniProtocolMessageToAgency: for<'b> Func<&'b mini_protocol::Message<MP>, Output = bool>,
{
    #[inline]
    fn call_once(self, (_, task_state): ReceiverState<'a, MP>) -> Self::Output {
        let message = match self.decoder.decode_with(&mut DecodeContext(None)) {
            Ok(m) => m,
            Err(e) => return FeedResult::Error(MuxError::Decode(e)),
        };

        let server_has_agency_next = MiniProtocolMessageToAgency::call(&message);
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

enum MessageToAgency {}
impl<M: Message> TypeMap<&M> for MessageToAgency {
    type Output = bool;
}
impl<M: Message> Func<&M> for MessageToAgency {
    fn call(_: &M) -> Self::Output {
        <M::ToState as State>::Agency::SERVER
    }
}

enum StateMessageToAgency {}
impl<'a, SM> TypeMap<&'a SM> for StateMessageToAgency
where
    SM: ToRef<'a>,
    Fold<MessageToAgency, bool>: Func<SM::Output, Output = bool>,
{
    type Output = bool;
}
impl<'a, SM> Func<&'a SM> for StateMessageToAgency
where
    SM: ToRef<'a>,
    Fold<MessageToAgency, bool>: Func<SM::Output, Output = bool>,
{
    fn call(i: &'a SM) -> Self::Output {
        Fold::call(i.to_ref())
    }
}

enum MiniProtocolMessageToAgency {}
impl<'a, MPM> TypeMap<&'a MPM> for MiniProtocolMessageToAgency
where
    MPM: ToRef<'a>,
    Fold<StateMessageToAgency, bool>: Func<<MPM as ToRef<'a>>::Output, Output = bool>,
{
    type Output = bool;
}
impl<'a, MPM> Func<&'a MPM> for MiniProtocolMessageToAgency
where
    MPM: ToRef<'a>,
    Fold<StateMessageToAgency, bool>: Func<<MPM as ToRef<'a>>::Output, Output = bool>,
{
    fn call(i: &'a MPM) -> Self::Output {
        Fold::call(i.to_ref())
    }
}
