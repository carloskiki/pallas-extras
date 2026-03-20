use crate::{
    Encoded,
    mux::{Egress, Ingress, header::Timestamp},
    traits::{
        message::{LazyDecode, Message, encode_message},
        mini_protocol::{self, MiniProtocol},
        protocol::Protocol,
        state::{self, Agency, State},
    },
    typefu::{
        FuncOnce,
        coproduct::{CoprodInjector, CoprodUninjector},
        map::{CMap, TypeMap},
    },
};
use tinycbor::{Decoder, Encode};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct Client<P, S> {
    sender: Sender<Egress<P>>,
    send_back: Sender<Ingress>,
    receiver: Receiver<Ingress>,
    _state: S,
}

impl<P, S> Client<P, S>
where
    P: Protocol,
    S: State<Agency = state::Client>,
{
    pub async fn send<M, IM>(self, message: &M) -> Option<Client<P, M::ToState>>
    where
        M: Message + Encode,
        S::Message: CoprodInjector<M, IM>,
    {
        // If the agency goes to the server, send a response_sender along to receive responses.
        let send_back =
            <<M::ToState as State>::Agency as Agency>::SERVER.then(|| self.send_back.clone());
        // TODO: Buffer pool.
        let mut encoded_message = Vec::new();
        encode_message(message, &mut encoded_message);

        self.sender
            .send(Egress {
                message: encoded_message,
                protocol: todo!(), // We need to store the protocol value for this somewhere, or derive
                // it from the mini-protocol (in which case we need MP type param).
                send_back,
            })
            .await
            .ok()?;

        Some(Client {
            sender: self.sender,
            send_back: self.send_back,
            receiver: self.receiver,
            _state: M::ToState::default(),
        })
    }
}

#[allow(private_bounds)]
impl<P, S> Client<P, S>
where
    P: Protocol,
    S: State<Agency = state::Server, Message: LazyDecode>,
    // From `Coproduct<M, ...>` to `Coproduct<Encoded<M>, ...>`.
    CMap<WrapEncoded>: TypeMap<<S as State>::Message>,
    // Decode `Coproduct<Encoded<M>, ...>`.
    <CMap<WrapEncoded> as TypeMap<<S as State>::Message>>::Output: LazyDecode,
    // From `Coproduct<Encoded<M>, ...>` to `Coproduct<(Encoded<M>, Client<P, M::ToState>), ...>`.
    CMap<MessageClientPair<P, S>>: FuncOnce<<S as State>::Message>,
{
    pub async fn receive<IS>(mut self) -> Result<Payload<P, S>, Error> {
        let Ingress { message, timestamp } = self.receiver.recv().await.ok_or(Error::Closed)?;

        let mut decoder = Decoder(&message);
        let mut visitor = decoder.array_visitor().map_err(|_| Error::Malformed)?;
        let tag: u64 = visitor
            .visit()
            .ok_or(Error::Malformed)?
            .map_err(|_| Error::Malformed)?;
        if !visitor.definite() {
            decoder.0 = decoder
                .0
                .split_last()
                .expect(
                    "message is guaranteed to be valid CBOR as received \
                    - if the array is indefinite, there is a break byte at the end",
                )
                .1
        }
        let message = message.slice_ref(decoder.0);
        let message_coproduct = LazyDecode::lazy_decode(message, tag).ok_or(Error::InvalidTag)?;

        Ok(Payload {
            timestamp,
            message: CMap(MessageClientPair { client: self }).call_once(message_coproduct),
        })
    }
}

#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum Error {
    /// message is valid CBOR but doesn't match the expected structure
    Malformed,
    /// the tag of the message is invalid
    InvalidTag,
    /// worker has been shut down
    Closed,
}

#[allow(private_bounds)]
pub struct Payload<P, S: State>
where
    CMap<MessageClientPair<P, S>>: TypeMap<<S as State>::Message>,
{
    pub timestamp: Timestamp,
    #[allow(private_interfaces)]
    pub message: NextClient<P, S>,
}

type NextClient<P, S> = <CMap<MessageClientPair<P, S>> as TypeMap<<S as State>::Message>>::Output;

/// Wrap a something into `Encoded` type.
enum WrapEncoded {}
impl<T> TypeMap<T> for WrapEncoded {
    type Output = Encoded<T>;
}


struct MessageClientPair<P, S>
where
    S: State,
{
    client: Client<P, S>,
}
impl<P, S, M> TypeMap<Encoded<M>> for MessageClientPair<P, S>
where
    M: Message,
    S: State,
{
    type Output = (Encoded<M>, Client<P, M::ToState>);
}
impl<P, S, M> FuncOnce<Encoded<M>> for MessageClientPair<P, S>
where
    M: Message,
    S: State,
{
    fn call_once(self, input: Encoded<M>) -> Self::Output {
        let MessageClientPair {
            client:
                Client {
                    receiver: response_receiver,
                    send_back: response_sender,
                    sender: request_sender,
                    ..
                },
        } = self;

        (
            input,
            Client {
                send_back: response_sender,
                receiver: response_receiver,
                sender: request_sender,
                _state: M::ToState::default(),
            },
        )
    }
}
