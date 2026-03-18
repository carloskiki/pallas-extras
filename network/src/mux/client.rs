use super::{MiniProtocolSendBundle, MuxError};
use crate::{
    mux::{Request, Response},
    traits::{
        message::{Message, encode_message},
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
use tokio::sync::mpsc::{Receiver, Sender};

#[allow(private_bounds)]
pub struct Client<P, S>
where
    P: Protocol,
    S: State,
{
    request_sender: Sender<Request<P>>,
    response_sender: Sender<Response>,
    response_receiver: Receiver<Response>,
    _state: S,
}

impl<P, S> Client<P, S>
where
    P: Protocol,
    S: State<Agency = state::Client>,
{
    pub async fn send<M, IM>(mut self, message: &M) -> Option<Client<P, M::ToState>>
    where
        M: Message,
        S::Message: CoprodInjector<M, IM>,
    {
        // If the agency goes to the server, send a response_sender along to receive responses.
        let send_back =
            <<M::ToState as State>::Agency as Agency>::SERVER.then(|| self.response_sender.clone());
        // TODO: Buffer pool.
        let mut encoded_message = Vec::new();
        encode_message(message, &mut encoded_message);

        self.request_sender
            .send(Request {
                message: encoded_message,
                protocol: todo!(), // We need to store the protocol value for this somewhere, or derive
                // it from the mini-protocol (in which case we need MP type param).
                send_back,
            })
            .await
            .ok()?;

        Some(Client {
            request_sender: self.request_sender,
            response_sender: self.response_sender,
            response_receiver: self.response_receiver,
            _state: M::ToState::default(),
        })
    }
}

#[allow(private_bounds)]
impl<P, S> Client<P, S>
where
    P: Protocol,
    S: State<Agency = state::Server>, {
    pub async fn receive<IS>(mut self) -> Option<NextClient<P, S>>
    where
        mini_protocol::Message<MP>: CoprodUninjector<S::Message, IS>,
    {
        let message = self.response_receiver.recv().await?;
        let state_message = received.uninject().or(Err(MuxError::InvalidPeerMessage))?;
            Ok(CMap(MessageClientPair { client: self }).call_once(state_message))
        },
    }
}

type NextClient<P, MP, S> =
    <CMap<MessageClientPair<P, MP, S>> as TypeMap<<S as State>::Message>>::Output;

#[doc(hidden)]
#[allow(private_bounds)]
pub struct MessageClientPair<P, MP, S>
where
    P: Protocol,
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    client: Client<P, MP, S>,
}
impl<P, MP, S, M> TypeMap<M> for MessageClientPair<P, MP, S>
where
    P: Protocol,
    MP: MiniProtocol,
    M: Message,
    CMap<state::Message>: TypeMap<MP::States>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    type Output = (M, Client<P, MP, M::ToState>);
}
// This isn't secure, one can map any message to a state, not just the one that the state allows.
// So this must be private.
impl<P, MP, S, M> FuncOnce<M> for MessageClientPair<P, MP, S>
where
    P: Protocol,
    MP: MiniProtocol,
    M: Message,
    CMap<state::Message>: TypeMap<MP::States>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    fn call_once(self, input: M) -> Self::Output {
        let MessageClientPair {
            client:
                Client {
                    task_handle,
                    response_receiver,
                    response_sender,
                    request_sender,
                    ..
                },
        } = self;

        (
            input,
            Client {
                task_handle,
                response_sender,
                response_receiver,
                request_sender,
                _state: M::ToState::default(),
            },
        )
    }
}
