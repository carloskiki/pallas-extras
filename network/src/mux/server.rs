use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{Receiver, Sender},
};

use crate::{
    traits::{
        message::Message,
        mini_protocol::{self, MiniProtocol},
        protocol::Protocol,
        state::{self, Client, State},
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
pub struct Server<P, MP, S>
where
    P: Protocol,
    MP: MiniProtocol,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
    CMap<state::Message>: TypeMap<MP::States>
{
    pub(super) task_handle: TaskHandle,
    pub(super) response_sender: Sender<ProtocolSendBundle<P>>,
    pub(super) request_receiver: Receiver<mini_protocol::Message<MP>>,
    pub(super) _state: S,
}

#[allow(private_bounds)]
impl<P, MP, S> Server<P, MP, S>
where
    P: Protocol,
    MP: MiniProtocol,
    S: State<Agency = Client>,
    CMap<state::Message>: TypeMap<MP::States>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
    CMap<MessageServerPair<P, MP, S>>: FuncOnce<S::Message>,
{
    #[allow(private_interfaces)]
    pub async fn receive<IM>(mut self) -> Result<NextServer<P, MP, S>, MuxError>
    where
        mini_protocol::Message<MP>: CoprodUninjector<S::Message, IM>,
    {
        let Some(received) = self.request_receiver.next().await else {
            return Err(catch_handle_error(self.task_handle));
        };
        let message = received.uninject().or(Err(MuxError::InvalidPeerMessage))?;
        Ok(CMap(MessageServerPair { server: self }).call_once(message))
    }
}

#[allow(private_bounds)]
impl<P, MP, S> Server<P, MP, S>
where
    P: Protocol,
    MP: MiniProtocol,
    S: State<Agency = state::Server>,
    CMap<state::Message>: TypeMap<MP::States>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
    CMap<MessageServerPair<P, MP, S>>: FuncOnce<S::Message>,
{
    pub async fn send<M, IM, IS, IMP>(
        mut self,
        message: M,
    ) -> Result<Server<P, MP, M::ToState>, MuxError>
    where
        M: Message,
        S::Message: CoprodInjector<M, IM>,
        mini_protocol::Message<MP>: CoprodInjector<S::Message, IS>,
        ProtocolSendBundle<P>: CoprodInjector<SendBundle<MP>, IMP>,
    {
        let state_message = S::Message::inject(message);
        let mini_protocol_message = mini_protocol::Message::<MP>::inject(state_message);
        let bundle = SendBundle {
            message: mini_protocol_message,
            send_back: None,
        };
        let protocol_bundle = ProtocolSendBundle::<P>::inject(bundle);
        let Ok(()) = self.response_sender.feed(protocol_bundle).await else {
            return Err(catch_handle_error(self.task_handle));
        };

        Ok(Server {
            task_handle: self.task_handle,
            response_sender: self.response_sender,
            request_receiver: self.request_receiver,
            _state: M::ToState::default(),
        })
    }
}

type NextServer<P, MP, S> =
    <CMap<MessageServerPair<P, MP, S>> as TypeMap<<S as State>::Message>>::Output;

struct MessageServerPair<P, MP, S>
where
    P: Protocol,
    MP: MiniProtocol,
    CMap<state::Message>: TypeMap<MP::States>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    server: Server<P, MP, S>,
}
impl<P, MP, S, M> TypeMap<M> for MessageServerPair<P, MP, S>
where
    P: Protocol,
    MP: MiniProtocol,
    M: Message,
    CMap<state::Message>: TypeMap<MP::States>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    type Output = (M, Server<P, MP, M::ToState>);
}
// This isn't secure, one can map any message to a state, not just the one that the state allows.
// So this must be private.
impl<P, MP, S, M> FuncOnce<M> for MessageServerPair<P, MP, S>
where
    P: Protocol,
    MP: MiniProtocol,
    M: Message,
    CMap<state::Message>: TypeMap<MP::States>,
    CMap<MiniProtocolSendBundle>: TypeMap<P>,
{
    fn call_once(self, input: M) -> Self::Output {
        let MessageServerPair {
            server:
                Server {
                    task_handle,
                    response_sender,
                    request_receiver,
                    ..
                },
        } = self;

        (
            input,
            Server {
                task_handle,
                response_sender,
                request_receiver,
                _state: M::ToState::default(),
            },
        )
    }
}
