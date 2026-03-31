use crate::{
    Encoded,
    agency::{Client, Server},
    message::Contains,
    mux::Handle,
};

use super::request::{Ids, Transactions};
use tinycbor::{Decode, Decoder};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Idle;

impl crate::State for Idle {
    const SIZE_LIMIT: usize = 5760;
    const TIMEOUT: ::std::time::Duration = std::time::Duration::MAX;
    type Agency = Server;
    type Message = Message;
}

pub enum Message {
    Transactions(
        Encoded<Transactions<'static>>,
        Handle<Client, <Transactions<'static> as crate::Message>::ToState>,
    ),
    Ids(
        Encoded<Ids<false>>,
        Handle<Client, <Ids<false> as crate::Message>::ToState>,
    ),
    IdsBlocking(
        Encoded<Ids<true>>,
        Handle<Client, <Ids<true> as crate::Message>::ToState>,
    ),
}

impl Contains<Transactions<'static>> for Message {}
impl Contains<Ids<false>> for Message {}
impl Contains<Ids<true>> for Message {}

impl crate::message::FromParts<Client> for Message {
    fn from_parts<S>(tag: u64, bytes: ::bytes::Bytes, handle: Handle<Client, S>) -> Option<Self> {
        match tag {
            <Transactions<'static> as crate::Message>::TAG => Some(Message::Transactions(
                Encoded::new(bytes),
                handle.transition(),
            )),
            <Ids<true> as crate::Message>::TAG
                if bool::decode(&mut Decoder(&bytes)) == Ok(true) =>
            {
                Some(Message::IdsBlocking(
                    Encoded::new(bytes),
                    handle.transition(),
                ))
            }
            <Ids<false> as crate::Message>::TAG => {
                Some(Message::Ids(Encoded::new(bytes), handle.transition()))
            }
            _ => None,
        }
    }
}
