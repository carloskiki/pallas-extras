use std::{
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
    task::Spawn,
};
use server::Server;

use crate::{
    traits::{
        mini_protocol::{self, MiniProtocol},
        protocol::{self, Protocol, UnknownProtocol},
        state,
    },
    typefu::{
        Func, FuncMany,
        constructor::Constructor,
        hlist::GetHead,
        map::{CMap, HMap, Identity, TypeMap},
        utilities::{Unzip, UnzipLeft, UnzipRight},
    },
};

pub mod client;
mod header;
mod reader_task;
pub mod server;

#[derive(Debug)]
pub enum MuxError {
    Io(io::Error),
    Protocol(UnknownProtocol),
    Decode(minicbor::decode::Error),
    Encode(minicbor::encode::Error<Infallible>),
    InvalidPeerMessage,
    AlreadyCaught,
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
        }
    }
}

impl Error for MuxError {}

#[allow(private_bounds)]
#[allow(private_interfaces)]
pub fn mux<P: Protocol>(
    bearer: impl AsyncRead + AsyncWrite,
    spawner: impl Spawn,
) -> <HMap<PairMaker<P>> as TypeMap<ServerReceivers<P>>>::Output
where
    HMap<Identity>: TypeMap<P>,
    HMap<ChannelPairMaker>: Constructor<protocol::List<P>>,
    UnzipLeft: TypeMap<ServerPairs<P>>,
    UnzipRight: TypeMap<ServerPairs<P>>,
    Unzip: Func<ServerPairs<P>, Output = (ServerSenders<P>, ServerReceivers<P>)>,
    HMap<PairMaker<P>>: FuncMany<ServerReceivers<P>>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    let (sender, receiver) = futures::channel::mpsc::channel(0);
    let (senders, receivers) = Unzip::call(HMap::<ChannelPairMaker>::construct());
    let task_handle = todo!();

    HMap(PairMaker {
        task_handle,
        sender,
    })
    .call_many(receivers)
}

type ServerPairs<P> = <HMap<ChannelPairMaker> as TypeMap<protocol::List<P>>>::Output;
type ServerSenders<P> =
    <UnzipLeft as TypeMap<<HMap<ChannelPairMaker> as TypeMap<protocol::List<P>>>::Output>>::Output;
type ServerReceivers<P> =
    <UnzipRight as TypeMap<<HMap<ChannelPairMaker> as TypeMap<protocol::List<P>>>::Output>>::Output;

enum ChannelPairMaker {}
impl<MP: MiniProtocol> TypeMap<MP> for ChannelPairMaker
where
    CMap<state::Message>: TypeMap<MP::States>,
{
    type Output = (Sender<mini_protocol::Message<MP>>, ReceiverWrapper<MP>);
}
impl<MP: MiniProtocol> Constructor<MP> for ChannelPairMaker
where
    CMap<crate::traits::state::Message>: TypeMap<MP::States>,
{
    fn construct() -> Self::Output {
        let (sender, rx) = futures::channel::mpsc::channel(MP::READ_BUFFER_SIZE);
        (sender, ReceiverWrapper { rx })
    }
}

type Pair<P, MP> = (
    Client<P, MP, <GetHead as TypeMap<<MP as MiniProtocol>::States>>::Output>,
    Server<P, MP, <GetHead as TypeMap<<MP as MiniProtocol>::States>>::Output>,
);
struct PairMaker<P>
where
    P: Protocol,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    task_handle: TaskHandle,
    sender: Sender<ProtocolSendBundle<P>>,
}
struct ReceiverWrapper<MP: MiniProtocol>
where
    CMap<state::Message>: TypeMap<MP::States>,
{
    rx: Receiver<mini_protocol::Message<MP>>,
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
    fn call_many(&self, ReceiverWrapper { rx }: ReceiverWrapper<MP>) -> Self::Output {
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
                state: Default::default(),
            },
        )
    }
}

struct SendBundle<MP>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>
{
    message: mini_protocol::Message<MP>,
    send_back: Option<UnboundedSender<mini_protocol::Message<MP>>>,
}
enum MiniProtocolSendBundle {}
impl<MP> TypeMap<MP> for MiniProtocolSendBundle
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>
{
    type Output = SendBundle<MP>;
}
type ProtocolSendBundle<P> = <CMap<MiniProtocolSendBundle> as TypeMap<P>>::Output;

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
    use crate::{protocol::NodeToNode, typefu::hlist::hlist_pat};

    use super::mux;

    #[test]
    fn create_mux() {
        let hlist_pat![
            (handshake_client, handshake_server),
            (chain_sync_client, chain_sync_server)
        ] = mux::<NodeToNode>(todo!(), todo!());
    }
}
