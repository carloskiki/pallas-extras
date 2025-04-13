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

use std::{collections::VecDeque, convert::Infallible, pin::pin};

use futures::{
    AsyncWrite, AsyncWriteExt, StreamExt,
    channel::mpsc::{Receiver, Sender, UnboundedSender},
};
use minicbor::{Decode, Encode};

use crate::{
    traits::{
        mini_protocol::{self, MiniProtocol},
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
    MiniProtocolSendBundle, MuxError, ProtocolSendBundle, SendBundle,
    header::{Header, ProtocolNumber},
};

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
        + CoproductFoldable<Poly<MessageAgency>, bool>,
    // Get a mutable reference to the mini protocol state
    ProtocolTaskState<P>: for<'a> ToMut<'a>,
    // Zip the message and state
    Zip<ProtocolSendBundle<P>>: for<'a> FuncOnce<<ProtocolTaskState<P> as ToMut<'a>>::Output>,
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
        let protocol = send_bundle.to_ref().map(Overwrite(protocol::List::<P>::default()));
        let server_sent = send_bundle.to_ref().fold(Poly(MessageAgency));

        CMap(ProcessBundle {
            encoder: &mut encoder,
        })
        .call_once(Zip(send_bundle).call_once(task_state.to_mut()))?;
        loop {
            if encoder.writer().len() >= u16::MAX as usize {
                break;
            }
            if let Ok(Some(send_bundle)) = rx.try_next() {
                if send_bundle.to_ref().map(Overwrite(protocol::List::<P>::default())) == protocol
                    && send_bundle.to_ref().fold(Poly(MessageAgency)) == server_sent
                {
                    CMap(ProcessBundle {
                        encoder: &mut encoder,
                    }).call_once(Zip(send_bundle).call_once(task_state.to_mut()))?;
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
                    server: server_sent,
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

type BundleRef<'a, P> = <ProtocolSendBundle<P> as ToRef<'a>>::Output;
type Zipped<'a, P> =
    <Zip<ProtocolSendBundle<P>> as TypeMap<<ProtocolTaskState<P> as ToMut<'a>>::Output>>::Output;

/// Getting the agency of the sender
struct MessageAgency;
impl<MP> TypeMap<&SendBundle<MP>> for MessageAgency
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
{
    type Output = bool;
}
impl<MP> Func<&SendBundle<MP>> for MessageAgency
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
    for<'a> &'a mini_protocol::Message<MP>: CoproductMappable<Overwrite<MP::States>, Output: CoproductFoldable<Poly<StateAgency>, bool>>,
{
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
    fn call(_: S) -> Self::Output {
        S::Agency::SERVER
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

impl<Input> TypeMap<&Input> for &'_ mut minicbor::Encoder<Vec<u8>>
where
    Input: Encode<()>,
{
    type Output = Result<(), minicbor::encode::Error<Infallible>>;
}

impl<Input> FuncOnce<&Input> for &'_ mut minicbor::Encoder<Vec<u8>>
where
    Input: Encode<()>,
{
    fn call_once(self, input: &Input) -> Self::Output {
        input.encode(self, &mut ())
    }
}

/// Process the bundle and return whether the bundle was sent by a server.
type Bundle<'a, MP> = (SendBundle<MP>, &'a mut TaskState<MP>);
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
    mini_protocol::Message<MP>: Encode<()>,
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

impl<MP> TypeMap<&MP> for &'_ mut minicbor::Decoder<'_>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States, Output: for<'a> Decode<'a, ()>>,
{
    type Output = Result<mini_protocol::Message<MP>, minicbor::decode::Error>;
}

impl<MP> FuncOnce<&MP> for &'_ mut minicbor::Decoder<'_>
where
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States, Output: for<'a> Decode<'a, ()>>,
{
    fn call_once(self, _: &MP) -> Self::Output {
        self.decode()
    }
}
