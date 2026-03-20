//! Implementation of the Multiplexer.
//!
//! The type requirements for the [`mux`] function may seem daunting, but the function's
//! documentation is quite clear.

// INVARIANT: The client must not keep agency after sending a message.

use std::{
    collections::VecDeque,
    convert::Infallible,
    io,
    task::{Poll, ready},
};

use bytes::Bytes;
use handle::Client;
use tokio::sync::mpsc::Sender;

use crate::{
    mux::header::Timestamp, traits::{
        mini_protocol::{self, MiniProtocol},
        protocol::{self, Protocol},
        state,
    }, typefu::{
        Func, FuncMany, FuncOnce, ToMut, ToRef,
        constructor::Constructor,
        hlist::GetHead,
        map::{CMap, Fold, HMap, Identity, Overwrite, TypeMap, Unzip, UnzipLeft, UnzipRight, Zip},
    }
};

// TODO: In client and server, ensure that the timeouts are checked.
// TODO: Check for cancel safety anywhere `select!` is used.
// TODO: Check for snoozing (pretty much anywhere async is used).

pub mod handle;
mod header;
mod task;

/// Errors that can occur while using the multiplexer.
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum MuxError<P> {
    /// IO error
    Io(#[from] io::Error),
    /// receive message for an unknown protocol
    UnknownProtocol(u16),
    /// received malformed message from a peer
    Malformed(Bytes),
    /// received a message that we weren't expecting in this state
    UnexpectedMessage(Bytes),
    /// receiving buffer for {protocol}[server: {server}] is full
    Full {
        protocol: P,
        server: bool,
    },
    /// all handles have been dropped
    Closed,
}

impl<P> From<u16> for MuxError<P> {
    fn from(value: u16) -> Self {
        MuxError::UnknownProtocol(value)
    }
}

struct Egress<P> {
    message: Vec<u8>,
    protocol: P,
    send_back: Option<Sender<Ingress>>,
}

struct Ingress {
    message: Bytes,
    timestamp: Timestamp,
}

pub fn mux() {}

// /// Create a multiplexer for the given protocol.
// ///
// /// __Don't pay attention to the type requirements!__ (unless you're interested :)).
// ///
// /// To use this function, one must provide a `bearer` (a connection to the peer), and a
// /// `spawner` (something that can spawn tasks). The function will return a [`Result`] containing
// /// all of the client-server pairs for each [`MiniProtocol`] in the [`Protocol`] provided. The type
// /// parameter `P` (the protocol) must be provided as the type argument.
// ///
// /// If `P` is not provided, one will cause a scary error message, something like "overflow
// /// evaluating the requirement ...". This happens because the compiler is stuck in an infinite loop
// /// trying to evaluate some requirements for an inferred type. The correct error should be: "type
// /// annotations needed".
// ///
// /// This crate provides two types that implement [`Protocol`]:
// /// - [`NodeToNode`](crate::protocol::NodeToNode): the protocol used for communication between
// ///   nodes.
// /// - [`NodeToClient`](crate::protocol::NodeToClient): the protocol used for communication between
// ///   nodes and clients.
// ///
// /// What this function returns in an [`HList`][HList]
// /// of `(Client, Server)` pairs. Each [`Client`] and [`Server`] to the spawned mux task.
// ///
// /// ## Example
// ///
// /// Using the `NodeToNode` protocol:
// /// ```
// /// use network::{hlist_pat, mux, protocol::NodeToNode};
// /// use futures::{io::AllowStdIo, executor::LocalPool};
// /// use std::net::TcpStream;
// ///
// /// let stream = TcpStream::connect("preview-node.play.dev.cardano.org:3001").unwrap();
// /// let pool = LocalPool::new();
// ///
// /// let hlist_pat![(handshake_client, _), (chain_sync_client, _), ...] =
// ///     mux::<NodeToNode>(AllowStdIo::new(stream), &pool.spawner()).unwrap();
// /// ```
// ///
// /// [HList]: https://beachape.com/blog/2016/10/23/rust-hlists-heterogenously-typed-list/
// #[allow(private_bounds)]
// #[allow(private_interfaces)]
// pub fn mux<P: Protocol>(
//     bearer: impl AsyncRead + AsyncWrite + Send + 'static,
//     spawner: &impl Spawn,
// ) -> Result<<HMap<PairMaker<P>> as TypeMap<ServerReceivers<P>>>::Output, SpawnError>
// where
//     HMap<Identity>: TypeMap<P, Output: Default>,
//     HMap<ChannelPairMaker>: Constructor<protocol::List<P>>,
//     HMap<MiniProtocolTaskState>: TypeMap<P, Output: Send>,
//     UnzipLeft: TypeMap<ServerPairs<P>>,
//     UnzipRight: TypeMap<ServerPairs<P>>,
//     Unzip: Func<ServerPairs<P>, Output = (ServerSenders<P>, ServerReceivers<P>)>,
//     HMap<PairMaker<P>>: FuncMany<ServerReceivers<P>>,
//     HMap<TaskStateMaker>: Func<ServerSenders<P>, Output = ProtocolTaskState<P>>,
//     CMap<MiniProtocolSendBundle>: TypeMap<P, Output: Send>,
//     // Get a ref to the send bundle
//     ProtocolSendBundle<P>: for<'a> ToRef<'a>,
//     // Get Protocol of send bundle + Get the agency of the sender
//     Overwrite<protocol::List<P>>: for<'a> FuncOnce<BundleRef<'a, P>, Output = P>,
//     Fold<MessageFromAgency, bool>: for<'a> Func<BundleRef<'a, P>, Output = bool>,
//     // Get a mutable reference to the mini protocol state
//     ProtocolTaskState<P>: for<'a> ToMut<'a>,
//     // Zip the message and state
//     for<'a> Zip<<ProtocolTaskState<P> as ToMut<'a>>::Output>: FuncOnce<ProtocolSendBundle<P>>,
//     // Process the bundle:
//     // - Encode the message in the buffer
//     // - Add the send_back in the queue if sent from client
//     for<'a, 'b, 'c> Fold<ProcessBundle<'a, 'b>, Result<(), minicbor::encode::Error<Infallible>>>:
//         FuncOnce<WriterZipped<'c, P>, Output = Result<(), minicbor::encode::Error<Infallible>>>,
//     // Get the task state for the protocol of the message received
//     for<'a> Zip<<ProtocolTaskState<P> as ToMut<'a>>::Output>: FuncOnce<P>,
//     // Decode and send the message to the correct handle.
//     for<'a, 'b, 'c> CMap<ProcessMessage<'a, 'b>>:
//         FuncOnce<ReaderZipped<'c, P>, Output: Future<Output = Result<(), MuxError>> + Send>,
// {
//     let (sender, receiver) = futures::channel::mpsc::channel(0);
//     let (senders, receivers) = Unzip::call(HMap::<ChannelPairMaker>::construct());
//     let task_state = HMap::<TaskStateMaker>::call(senders);
//     let task_handle = TaskHandle(Arc::new(Mutex::new(Some(
//         spawner.spawn_with_handle(task::task(bearer, receiver, task_state))?,
//     ))));
// 
//     Ok(HMap(PairMaker {
//         task_handle,
//         sender,
//     })
//     .call_many(receivers))
// }
// 
// type ServerPairs<P> = <HMap<ChannelPairMaker> as TypeMap<protocol::List<P>>>::Output;
// type ServerSenders<P> =
//     <UnzipLeft as TypeMap<<HMap<ChannelPairMaker> as TypeMap<protocol::List<P>>>::Output>>::Output;
// type ServerReceivers<P> =
//     <UnzipRight as TypeMap<<HMap<ChannelPairMaker> as TypeMap<protocol::List<P>>>::Output>>::Output;
// 
// #[doc(hidden)]
// pub enum ChannelPairMaker {}
// impl<MP: MiniProtocol> TypeMap<MP> for ChannelPairMaker
// where
//     CMap<state::Message>: TypeMap<MP::States>,
// {
//     type Output = (SenderWrapper<MP>, ReceiverWrapper<MP>);
// }
// impl<MP: MiniProtocol> Constructor<MP> for ChannelPairMaker
// where
//     CMap<crate::traits::state::Message>: TypeMap<MP::States>,
// {
//     fn construct() -> Self::Output {
//         let (sender, receiver) = futures::channel::mpsc::channel(MP::READ_BUFFER_SIZE);
//         (SenderWrapper { sender }, ReceiverWrapper { receiver })
//     }
// }
// 
// type Pair<P, MP> = (
//     Client<P, MP, <GetHead as TypeMap<<MP as MiniProtocol>::States>>::Output>,
//     Server<P, MP, <GetHead as TypeMap<<MP as MiniProtocol>::States>>::Output>,
// );
// #[doc(hidden)]
// #[allow(private_bounds)]
// pub struct PairMaker<P>
// where
//     P: Protocol,
//     CMap<MiniProtocolSendBundle>: TypeMap<P>,
// {
//     task_handle: TaskHandle,
//     sender: Sender<ProtocolSendBundle<P>>,
// }
// #[doc(hidden)]
// pub struct ReceiverWrapper<MP: MiniProtocol>
// where
//     CMap<state::Message>: TypeMap<MP::States>,
// {
//     receiver: Receiver<mini_protocol::Message<MP>>,
// }
// impl<P, MP> TypeMap<ReceiverWrapper<MP>> for PairMaker<P>
// where
//     P: Protocol,
//     CMap<MiniProtocolSendBundle>: TypeMap<P>,
//     CMap<state::Message>: TypeMap<MP::States>,
//     GetHead: TypeMap<MP::States>,
//     MP: MiniProtocol,
// {
//     type Output = Pair<P, MP>;
// }
// impl<P, MP> FuncMany<ReceiverWrapper<MP>> for PairMaker<P>
// where
//     P: Protocol,
//     CMap<MiniProtocolSendBundle>: TypeMap<P>,
//     MP: MiniProtocol,
//     CMap<state::Message>: TypeMap<MP::States>,
//     GetHead: TypeMap<MP::States>,
//     <GetHead as TypeMap<MP::States>>::Output: Default,
// {
//     fn call_many(&self, ReceiverWrapper { receiver: rx }: ReceiverWrapper<MP>) -> Self::Output {
//         let (response_sender, response_receiver) = futures::channel::mpsc::unbounded();
//         (
//             Client {
//                 task_handle: self.task_handle.clone(),
//                 request_sender: self.sender.clone(),
//                 response_sender,
//                 response_receiver,
//                 _state: Default::default(),
//             },
//             Server {
//                 task_handle: self.task_handle.clone(),
//                 response_sender: self.sender.clone(),
//                 request_receiver: rx,
//                 _state: Default::default(),
//             },
//         )
//     }
// }
// 
// enum TaskStateMaker {}
// #[doc(hidden)]
// pub struct SenderWrapper<MP: MiniProtocol>
// where
//     CMap<state::Message>: TypeMap<MP::States>,
// {
//     sender: Sender<mini_protocol::Message<MP>>,
// }
// impl<MP> TypeMap<SenderWrapper<MP>> for TaskStateMaker
// where
//     MP: MiniProtocol,
//     CMap<state::Message>: TypeMap<MP::States>,
// {
//     type Output = TaskState<MP>;
// }
// impl<MP> Func<SenderWrapper<MP>> for TaskStateMaker
// where
//     MP: MiniProtocol,
//     CMap<state::Message>: TypeMap<MP::States>,
// {
//     fn call(SenderWrapper { sender }: SenderWrapper<MP>) -> Self::Output {
//         TaskState {
//             server_send_back: sender,
//             client_send_backs: VecDeque::new(),
//         }
//     }
// }
// 
// enum MiniProtocolTaskState {}
// impl<MP> TypeMap<MP> for MiniProtocolTaskState
// where
//     MP: MiniProtocol,
//     CMap<state::Message>: TypeMap<MP::States>,
// {
//     type Output = TaskState<MP>;
// }
// type ProtocolTaskState<P> = <HMap<MiniProtocolTaskState> as TypeMap<P>>::Output;
// 
// #[cfg(test)]
// mod tests {
//     use crate::{
//         mux::{FuncOnce, MiniProtocolSendBundle, ProtocolSendBundle, ToRef, task::BundleRef},
//         protocol::NodeToNode,
//         traits::protocol,
//         typefu::map::{CMap, HMap, Identity, Overwrite, TypeMap},
//     };
// 
//     #[test]
//     fn create_mux() {
//         fn test<P>()
//         where
//             CMap<MiniProtocolSendBundle>: TypeMap<P>,
//             ProtocolSendBundle<P>: for<'a> ToRef<'a>,
//             HMap<Identity>: TypeMap<P>,
//             Overwrite<protocol::List<P>>: for<'a> FuncOnce<BundleRef<'a, P>, Output = P>,
//         {
//         }
//         test::<NodeToNode>();
//     }
// }
