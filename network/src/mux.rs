use std::{
    collections::VecDeque,
    convert::Infallible,
    error::Error,
    fmt::Display,
    io,
    pin::pin,
    sync::{Arc, Mutex},
    task::Poll,
};

use client::Client;
use futures::{
    AsyncRead, AsyncWrite,
    channel::mpsc::{Receiver, Sender, UnboundedSender},
    future::RemoteHandle,
    task::{Spawn, SpawnError, SpawnExt},
};
use server::Server;
use task::{
    BundleRef, MessageFromAgency, ProcessBundle, ProcessMessage, ReaderZipped, WriterZipped,
};

use crate::{
    traits::{
        mini_protocol::{self, MiniProtocol},
        protocol::{self, Protocol, UnknownProtocol},
        state,
    },
    typefu::{
        Func, FuncMany, FuncOnce, ToMut, ToRef,
        constructor::Constructor,
        hlist::GetHead,
        map::{CMap, Fold, HMap, Identity, Overwrite, TypeMap, Unzip, UnzipLeft, UnzipRight, Zip},
    },
};

// TODO: In client and server, ensure that the timeouts are checked.

pub mod client;
mod header;
pub mod server;
mod task;

#[derive(Debug)]
pub enum MuxError {
    /// An IO error occurred.
    Io(io::Error),
    /// The message received contained an unknown protocol.
    Protocol(UnknownProtocol),
    /// An error occurred while decoding a message.
    Decode(minicbor::decode::Error),
    /// An error occurred while encoding a message.
    Encode(minicbor::encode::Error<Infallible>),
    /// The peer sent a message that is invalid for the current state.
    InvalidPeerMessage,
    /// The error was already caught by another handle.
    AlreadyCaught,
    /// The sender or receiver from a `Server` or `Client` handle was dropped.
    HandleDropped,
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
            Self::AlreadyCaught => {
                write!(f, "The MUX task error was already caught by another handle")
            }
            Self::InvalidPeerMessage => {
                write!(f, "Peer sent a message invalid for the current state")
            }
            Self::HandleDropped => {
                write!(
                    f,
                    "A `Server` or `Client` handle was dropped while it was still awaiting messages."
                )
            }
        }
    }
}

impl Error for MuxError {}

#[allow(private_bounds)]
#[allow(private_interfaces)]
pub fn mux<P: Protocol>(
    bearer: impl AsyncRead + AsyncWrite + Send + 'static,
    spawner: &impl Spawn,
) -> Result<<HMap<PairMaker<P>> as TypeMap<ServerReceivers<P>>>::Output, SpawnError>
where
    HMap<Identity>: TypeMap<P, Output: Default>,
    HMap<ChannelPairMaker>: Constructor<protocol::List<P>>,
    HMap<MiniProtocolTaskState>: TypeMap<P, Output: Send>,
    UnzipLeft: TypeMap<ServerPairs<P>>,
    UnzipRight: TypeMap<ServerPairs<P>>,
    Unzip: Func<ServerPairs<P>, Output = (ServerSenders<P>, ServerReceivers<P>)>,
    HMap<PairMaker<P>>: FuncMany<ServerReceivers<P>>,
    HMap<TaskStateMaker>: Func<ServerSenders<P>, Output = ProtocolTaskState<P>>,
    CMap<MiniProtocolSendBundle>: TypeMap<P, Output: Send>,
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
    for<'a, 'b, 'c> CMap<ProcessMessage<'a, 'b>>:
        FuncOnce<ReaderZipped<'c, P>, Output: Future<Output = Result<(), MuxError>> + Send>,
{
    let (sender, receiver) = futures::channel::mpsc::channel(0);
    let (senders, receivers) = Unzip::call(HMap::<ChannelPairMaker>::construct());
    let task_state = HMap::<TaskStateMaker>::call(senders);
    let task_handle = Arc::new(Mutex::new(Some(
        spawner.spawn_with_handle(task::task(bearer, receiver, task_state))?,
    )));

    Ok(HMap(PairMaker {
        task_handle,
        sender,
    })
    .call_many(receivers))
}

type ServerPairs<P> = <HMap<ChannelPairMaker> as TypeMap<protocol::List<P>>>::Output;
type ServerSenders<P> =
    <UnzipLeft as TypeMap<<HMap<ChannelPairMaker> as TypeMap<protocol::List<P>>>::Output>>::Output;
type ServerReceivers<P> =
    <UnzipRight as TypeMap<<HMap<ChannelPairMaker> as TypeMap<protocol::List<P>>>::Output>>::Output;

#[doc(hidden)]
pub enum ChannelPairMaker {}
impl<MP: MiniProtocol> TypeMap<MP> for ChannelPairMaker
where
    CMap<state::Message>: TypeMap<MP::States>,
{
    type Output = (SenderWrapper<MP>, ReceiverWrapper<MP>);
}
impl<MP: MiniProtocol> Constructor<MP> for ChannelPairMaker
where
    CMap<crate::traits::state::Message>: TypeMap<MP::States>,
{
    fn construct() -> Self::Output {
        let (sender, receiver) = futures::channel::mpsc::channel(MP::READ_BUFFER_SIZE);
        (SenderWrapper { sender }, ReceiverWrapper { receiver })
    }
}

type Pair<P, MP> = (
    Client<P, MP, <GetHead as TypeMap<<MP as MiniProtocol>::States>>::Output>,
    Server<P, MP, <GetHead as TypeMap<<MP as MiniProtocol>::States>>::Output>,
);
#[doc(hidden)]
#[allow(private_bounds)]
pub struct PairMaker<P>
where
    P: Protocol,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    task_handle: TaskHandle,
    sender: Sender<ProtocolSendBundle<P>>,
}
#[doc(hidden)]
pub struct ReceiverWrapper<MP: MiniProtocol>
where
    CMap<state::Message>: TypeMap<MP::States>,
{
    receiver: Receiver<mini_protocol::Message<MP>>,
}
impl<P, MP> TypeMap<ReceiverWrapper<MP>> for PairMaker<P>
where
    P: Protocol,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
    CMap<state::Message>: TypeMap<MP::States>,
    GetHead: TypeMap<MP::States>,
    MP: MiniProtocol,
{
    type Output = Pair<P, MP>;
}
impl<P, MP> FuncMany<ReceiverWrapper<MP>> for PairMaker<P>
where
    P: Protocol,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
    GetHead: TypeMap<MP::States>,
    <GetHead as TypeMap<MP::States>>::Output: Default,
{
    fn call_many(&self, ReceiverWrapper { receiver: rx }: ReceiverWrapper<MP>) -> Self::Output {
        let (response_sender, response_receiver) = futures::channel::mpsc::unbounded();
        (
            Client {
                task_handle: self.task_handle.clone(),
                request_sender: self.sender.clone(),
                response_sender,
                response_receiver,
                _state: Default::default(),
            },
            Server {
                task_handle: self.task_handle.clone(),
                response_sender: self.sender.clone(),
                request_receiver: rx,
                _state: Default::default(),
            },
        )
    }
}

struct SendBundle<MP>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    message: mini_protocol::Message<MP>,
    send_back: Option<UnboundedSender<mini_protocol::Message<MP>>>,
}
enum MiniProtocolSendBundle {}
impl<MP> TypeMap<MP> for MiniProtocolSendBundle
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    type Output = SendBundle<MP>;
}
type ProtocolSendBundle<P> = <CMap<MiniProtocolSendBundle> as TypeMap<P>>::Output;

enum TaskStateMaker {}
#[doc(hidden)]
pub struct SenderWrapper<MP: MiniProtocol>
where
    CMap<state::Message>: TypeMap<MP::States>,
{
    sender: Sender<mini_protocol::Message<MP>>,
}
impl<MP> TypeMap<SenderWrapper<MP>> for TaskStateMaker
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    type Output = TaskState<MP>;
}
impl<MP> Func<SenderWrapper<MP>> for TaskStateMaker
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    fn call(SenderWrapper { sender }: SenderWrapper<MP>) -> Self::Output {
        TaskState {
            server_send_back: sender,
            client_send_backs: VecDeque::new(),
        }
    }
}

struct TaskState<MP>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    server_send_back: Sender<mini_protocol::Message<MP>>,
    client_send_backs: VecDeque<UnboundedSender<mini_protocol::Message<MP>>>,
}
enum MiniProtocolTaskState {}
impl<MP> TypeMap<MP> for MiniProtocolTaskState
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    type Output = TaskState<MP>;
}
type ProtocolTaskState<P> = <HMap<MiniProtocolTaskState> as TypeMap<P>>::Output;

type TaskHandle = Arc<Mutex<Option<RemoteHandle<Result<Infallible, MuxError>>>>>;
fn catch_handle_error(handle: TaskHandle) -> MuxError {
    let Some(lock) = handle.lock().unwrap().take() else {
        return MuxError::AlreadyCaught;
    };
    let pinned_lock = pin!(lock);
    let Poll::Ready(Err(e)) =
        pinned_lock.poll(&mut std::task::Context::from_waker(std::task::Waker::noop()))
    else {
        unreachable!("Handler misbehaved, but it is still running...");
    };
    e
}

#[cfg(test)]
mod tests {
    use futures::{executor::LocalPool, io::Cursor, task::Spawn};

    use crate::{hlist_pat, mux::{task::BundleRef, FuncOnce, MiniProtocolSendBundle, ProtocolSendBundle, ToRef}, protocol::NodeToNode, traits::protocol::{self, Protocol}, typefu::map::{CMap, HMap, Identity, Overwrite, TypeMap}};

    use super::mux;

    #[test]
    fn create_mux() {
        fn test<P>()
            where
                CMap<MiniProtocolSendBundle>: TypeMap<P>,
                ProtocolSendBundle<P>: for<'a> ToRef<'a>,
                HMap<Identity>: TypeMap<P>,
                Overwrite<protocol::List<P>>: for<'a> FuncOnce<BundleRef<'a, P>, Output = P>,
        {}
        test::<NodeToNode>();
    }
}
