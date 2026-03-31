use crate::{
    NetworkMagic, State, agency::Server, handshake::VersionTable, message::Contains, state::Done,
};
use tinycbor_derive::{CborLen, Decode, Encode};

pub struct Confirm<VD>(std::marker::PhantomData<VD>);

impl<VD> State for Confirm<VD> {
    const SIZE_LIMIT: usize = 5760;
    const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
    type Agency = Server;
    type Message = Message<VD>;
}

pub enum Message<VD> {
    Accept(
        crate::Encoded<Accept<VD>>,
        crate::mux::Handle<Server, <Accept<VD> as crate::Message>::ToState>,
    ),
    Refuse(
        crate::Encoded<Refuse<'static>>,
        crate::mux::Handle<Server, <Refuse<'static> as crate::Message>::ToState>,
    ),
    Reply(
        crate::Encoded<Reply<VD>>,
        crate::mux::Handle<Server, <Reply<VD> as crate::Message>::ToState>,
    ),
}

impl<VD> Contains<Accept<VD>> for Message<VD> {}
impl<VD> Contains<Refuse<'static>> for Message<VD> {}
impl<VD> Contains<Reply<VD>> for Message<VD> {}

impl<VD> crate::message::FromParts<Server> for Message<VD> {
    fn from_parts<S>(
        tag: u64,
        bytes: ::bytes::Bytes,
        handle: crate::mux::Handle<Server, S>,
    ) -> Option<Self> {
        match tag {
            <Accept<VD> as crate::Message>::TAG => Some(Message::Accept(
                crate::Encoded::new(bytes),
                handle.transition(),
            )),
            <Refuse<'static> as crate::Message>::TAG => Some(Message::Refuse(
                crate::Encoded::new(bytes),
                handle.transition(),
            )),
            <Reply<VD> as crate::Message>::TAG => Some(Message::Reply(
                crate::Encoded::new(bytes),
                handle.transition(),
            )),
            _ => None,
        }
    }
}

mod accept_version {
    use crate::handshake::Version;

    use super::*;

    #[derive(Debug, Encode, Decode, CborLen)]
    #[cbor(
        naked,
        decode_bound = "D: tinycbor::Decode<'_>",
        encode_bound = "D: tinycbor::Encode",
        len_bound = "D: tinycbor::CborLen"
    )]
    pub struct Accept<D>(pub Version, pub D);
}
pub use accept_version::Accept;

impl<D> crate::Message for Accept<D> {
    const TAG: u64 = 1;

    type ToState = Done;
}

mod refuse {
    use super::*;
    use crate::handshake::Version;

    #[derive(Debug, Clone, Encode, Decode, CborLen)]
    pub enum Refuse<'a> {
        #[n(0)]
        VersionMismatch(Vec<Version>),
        #[n(1)]
        HandshakeDecodeError(Version, &'a str),
        #[n(2)]
        Refused(Version, &'a str),
    }
}
pub use refuse::Refuse;

impl crate::Message for Refuse<'static> {
    const TAG: u64 = 2;

    type ToState = Done;
}

#[derive(Debug, Encode, Decode, CborLen)]
#[cbor(
    naked,
    decode_bound = "D: tinycbor::Decode<'_>",
    encode_bound = "D: tinycbor::Encode",
    len_bound = "D: tinycbor::CborLen"
)]
pub struct Reply<D>(pub VersionTable<D>);

impl<D> crate::Message for Reply<D> {
    const TAG: u64 = 3;

    type ToState = Done;
}
