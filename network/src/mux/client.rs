use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{Sender, UnboundedReceiver, UnboundedSender},
};

use crate::{
    traits::{
        message::Message,
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

use super::{
    MiniProtocolSendBundle, MuxError, ProtocolSendBundle, SendBundle, TaskHandle,
    catch_handle_error,
};

#[allow(private_bounds)]
pub struct Client<P, MP, S>
where
    P: Protocol,
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    pub(super) task_handle: TaskHandle,
    pub(super) request_sender: Sender<ProtocolSendBundle<P>>,
    pub(super) response_sender: UnboundedSender<mini_protocol::Message<MP>>,
    pub(super) response_receiver: UnboundedReceiver<mini_protocol::Message<MP>>,
    pub(super) _state: S,
}

impl<P, MP, S> Client<P, MP, S>
where
    P: Protocol,
    MP: MiniProtocol,
    S: State<Agency = state::Client>,
    CMap<state::Message>: TypeMap<MP::States>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    pub async fn send<M, IM, IS, IMP>(
        mut self,
        message: M,
    ) -> Result<Client<P, MP, M::ToState>, MuxError>
    where
        M: Message,
        S::Message: CoprodInjector<M, IM>,
        mini_protocol::Message<MP>: CoprodInjector<S::Message, IS>,
        ProtocolSendBundle<P>: CoprodInjector<SendBundle<MP>, IMP>,
    {
        // If the agency goes to the server, send a response_sender along to receive responses.
        let send_back =
            <<M::ToState as State>::Agency as Agency>::SERVER.then(|| self.response_sender.clone());

        let state_message = S::Message::inject(message);
        let mini_protocol_message = mini_protocol::Message::<MP>::inject(state_message);
        let bundle = SendBundle {
            message: mini_protocol_message,
            send_back,
        };

        let protocol_bundle = ProtocolSendBundle::<P>::inject(bundle);

        if let Err(_) = self.request_sender.feed(protocol_bundle).await {
            return Err(catch_handle_error(self.task_handle));
        }

        Ok(Client {
            task_handle: self.task_handle,
            request_sender: self.request_sender,
            response_sender: self.response_sender,
            response_receiver: self.response_receiver,
            _state: M::ToState::default(),
        })
    }
}

impl<P, MP, S> Client<P, MP, S>
where
    P: Protocol,
    MP: MiniProtocol,
    S: State<Agency = state::Server>,
    CMap<state::Message>: TypeMap<MP::States>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
    CMap<MessageClientPair<P, MP, S>>: FuncOnce<S::Message>,
{
    pub async fn receive<IS>(mut self) -> Result<NextClient<P, MP, S>, MuxError>
    where
        mini_protocol::Message<MP>: CoprodUninjector<S::Message, IS>,
    {
        let Some(received) = self.response_receiver.next().await else {
            return Err(catch_handle_error(self.task_handle));
        };
        let state_message = received.uninject().or(Err(MuxError::InvalidPeerMessage))?;
        Ok(CMap(MessageClientPair { client: self }).call_once(state_message))
    }
}

type NextClient<P, MP, S> =
    <CMap<MessageClientPair<P, MP, S>> as TypeMap<<S as State>::Message>>::Output;

struct MessageClientPair<P, MP, S>
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
