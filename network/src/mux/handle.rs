use crate::{
    Agency, Message, State,
    agency::{Client, Server},
    message::FromParts,
    mux::{Egress, Ingress, header::ProtocolNumber, task},
    state::InitialState,
};
use bytes::BytesMut;
use std::marker::PhantomData;
use tinycbor::Encode;
use tokio::sync::mpsc::{self, Receiver, Sender};

// TODO:
// - implement timeouts and size limits.

pub(crate) fn components<S: InitialState>(
    sender: mpsc::Sender<Egress>,
) -> (Handle<Client, S>, Handle<Server, S>, task::State) {
    let (client_send_back, receiver) = mpsc::channel(S::INGRESS_BUFFER_SIZE);
    let client_handle = Handle {
        sender: sender.clone(),
        buffer: BytesMut::new(),
        receiver,
        protocol_id: S::PROTOCOL_ID,
        _phantom: PhantomData,
    };

    let (server_send_back, receiver) = mpsc::channel(S::INGRESS_BUFFER_SIZE);
    let server_handle = Handle {
        sender,
        buffer: BytesMut::new(),
        receiver,
        protocol_id: S::PROTOCOL_ID,
        _phantom: PhantomData,
    };

    let state = task::State {
        read_buffer: BytesMut::new(),
        read_position: 0,
        read_state: tinycbor::stream::Any::default(),
        server_send_back,
        client_send_back,
    };

    (client_handle, server_handle, state)
}

pub struct Handle<A, S> {
    sender: Sender<Egress>,
    receiver: Receiver<Ingress>,
    buffer: BytesMut,
    protocol_id: u16,
    _phantom: PhantomData<(S, A)>,
}

impl<A, S> Handle<A, S> {
    pub(crate) fn transition<NS>(self) -> Handle<A, NS> {
        Handle {
            sender: self.sender,
            receiver: self.receiver,
            buffer: self.buffer,
            protocol_id: self.protocol_id,
            _phantom: PhantomData,
        }
    }
}

impl<A, S> Handle<A, S>
where
    A: Agency,
    S: State<Agency = A>,
{
    pub async fn send<M>(mut self, message: &M) -> Option<Handle<A, M::ToState>>
    where
        M: Message + Encode,
        S::Message: crate::message::Contains<M>,
    {
        self.sender
            .send(Egress::new(
                message,
                &mut self.buffer,
                ProtocolNumber::new(self.protocol_id, A::SERVER),
            ))
            .await
            .ok()?;

        Some(self.transition())
    }
}

impl<A, S> Handle<A, S>
where
    A: Agency,
    S: State<Agency = A::Inverse>,
    S::Message: FromParts<A>,
{
    pub async fn receive(mut self) -> Result<S::Message, Error> {
        let Ingress { message } = self.receiver.recv().await.ok_or(Error::Closed)?;

        let mut decoder = tinycbor::Decoder(&message);
        let (tag, definite) = {
            let mut visitor = decoder.array_visitor().map_err(|_| Error::Malformed)?;
            let tag = visitor
                .visit::<u64>()
                .ok_or(Error::Malformed)?
                .map_err(|_| Error::Malformed)?;
            (tag, visitor.definite())
        };

        let mut payload = message.slice_ref(decoder.0);
        if !definite {
            let Some((0xff, without_break)) = payload.split_last() else {
                return Err(Error::Malformed);
            };
            payload = message.slice_ref(without_break);
        }

        S::Message::from_parts(tag, payload, self).ok_or(Error::InvalidTag)
    }
}

#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum Error {
    /// the message is not a CBOR array containing a numeric tag
    Malformed,
    /// the tag of the message is invalid
    InvalidTag,
    /// worker has been shut down
    Closed,
}
