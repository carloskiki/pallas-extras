use super::state::Confirm;
use crate::{
    NetworkMagic,
    traits::{message::Message, state::Done},
};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Encode, Decode)]
#[cbor(
    naked,
    decode_bound = "D: tinycbor::Decode<'_>",
    encode_bound = "D: tinycbor::Encode",
    len_bound = "D: tinycbor::CborLen"
)]
pub struct ProposeVersions<D>(pub VersionTable<D>);

impl<D> Message for ProposeVersions<D> {
    const SIZE_LIMIT: usize = 5760;
    const TAG: u64 = 0;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Confirm<D>;
}

mod accept_version {
    use super::*;

    #[derive(Debug, Encode, Decode, CborLen)]
    #[cbor(
        naked,
        decode_bound = "D: tinycbor::Decode<'_>",
        encode_bound = "D: tinycbor::Encode",
        len_bound = "D: tinycbor::CborLen"
    )]
    pub struct AcceptVersion<D>(pub Version, pub D);
}
pub use accept_version::AcceptVersion;

impl<D> Message for AcceptVersion<D> {
    const SIZE_LIMIT: usize = 5760;
    const TAG: u64 = 1;
    const ELEMENT_COUNT: u64 = 2;

    type ToState = Done;
}

#[derive(Debug, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct Refuse<'a>(pub RefuseReason<'a>);

impl Message for Refuse<'static> {
    const SIZE_LIMIT: usize = 5760;
    const TAG: u64 = 2;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Done;
}

#[derive(Debug, Encode, Decode, CborLen)]
#[cbor(naked, decode_bound = "D: tinycbor::Decode<'_>")]
pub struct QueryReply<D>(pub VersionTable<D>);

impl<D> Message for QueryReply<D> {
    const SIZE_LIMIT: usize = 5760;
    const TAG: u64 = 3;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Done;
}

#[derive(Debug, Encode, Decode, CborLen)]
#[cbor(naked, decode_bound = "D: tinycbor::Decode<'_>")]
pub struct VersionTable<D> {
    pub versions: Vec<(Version, D)>,
}

mod node_to_node {
    use super::*;
    use tinycbor::{CborLen, Decode, Encode};

    #[derive(Debug, Encode, Decode, CborLen)]
    pub struct VersionData {
        pub network_magic: NetworkMagic,
        pub diffusion_mode: bool,
        #[cbor(with = "BoolU")]
        pub peer_sharing: bool,
        pub query: bool,
    }

    struct BoolU(bool);

    impl Encode for BoolU {
        fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
            (self.0 as u64).encode(e)
        }
    }

    impl Decode<'_> for BoolU {
        type Error = tinycbor::primitive::Error;

        fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
            let value = u64::decode(d)?;
            if value == 1 {
                Ok(Self(true))
            } else if value == 0 {
                Ok(Self(false))
            } else {
                Err(tinycbor::primitive::Error::InvalidHeader)
            }
        }
    }

    impl CborLen for BoolU {
        fn cbor_len(&self) -> usize {
            1
        }
    }
}

mod node_to_client {
    use super::*;

    #[derive(Debug, Encode, Decode)]
    pub struct VersionData {
        pub network_magic: NetworkMagic,
        pub query: bool,
    }
}

mod refuse_reason {
    use super::*;

    #[derive(Debug, Encode, Decode)]
    pub enum RefuseReason<'a> {
        #[n(0)]
        VersionMismatch(Vec<Version>),
        #[n(1)]
        HandshakeDecodeError(Version, &'a str),
        #[n(2)]
        Refused(Version, &'a str),
    }
}
pub use refuse_reason::RefuseReason;

pub type Version = u16;
