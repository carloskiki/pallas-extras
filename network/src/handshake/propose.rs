use std::time::Duration;

use tinycbor_derive::{CborLen, Decode, Encode};

use crate::{Message, State, agency::Client, handshake::VersionTable, message};

pub struct Propose<VD>(std::marker::PhantomData<VD>);

impl<VD> State for Propose<VD> {
    const SIZE_LIMIT: usize = 5760;
    const TIMEOUT: Duration = Duration::from_secs(10);
    type Agency = Client;

    type Message = message::Single<Client, Versions<VD>>;
}

impl<VD> crate::state::InitialState for Propose<VD> {
    const PROTOCOL_ID: u16 = 0;
    const INGRESS_BUFFER_SIZE: usize = 1;
}

#[derive(Debug, Encode, Decode, CborLen)]
#[cbor(
    naked,
    decode_bound = "D: tinycbor::Decode<'_>",
    encode_bound = "D: tinycbor::Encode",
    len_bound = "D: tinycbor::CborLen"
)]
pub struct Versions<D>(pub VersionTable<D>);

impl<D> Message for Versions<D> {
    const TAG: u64 = 0;

    type ToState = super::Confirm<D>;
}

