use std::borrow::Cow;

use minicbor::{Decode, Encode, Encoder, encode};

use crate::{traits::{message::Message, state::Done}, NetworkMagic};

use super::state::Confirm;

#[derive(Debug, Encode, Decode)]
#[cbor(transparent)]
pub struct ProposeVersions<D>(pub VersionTable<D>);

impl<D> Message for ProposeVersions<D>
where
    D: Encode<()> + for<'a> Decode<'a, ()> + 'static,
{
    const SIZE_LIMIT: usize = 5760;
    const TAG: u8 = 0;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Confirm<D>;
}

#[derive(Debug)]
pub struct AcceptVersion<D>(pub Version, pub D);

impl<C, D: Encode<C>> Encode<C> for AcceptVersion<D> {
    fn encode<W: encode::Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        e.u16(self.0)?;
        e.encode_with(&self.1, ctx)?.ok()
    }
}

impl<'a, C, D: Decode<'a, C>> Decode<'a, C> for AcceptVersion<D> {
    fn decode(d: &mut minicbor::Decoder<'a>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        Ok(Self(d.u16()?, d.decode_with(ctx)?))
    }
}

impl<D> Message for AcceptVersion<D>
where
    D: Encode<()> + for<'a> Decode<'a, ()> + 'static,
{
    const SIZE_LIMIT: usize = 5760;
    const TAG: u8 = 1;
    const ELEMENT_COUNT: u64 = 2;

    type ToState = Done;
}

#[derive(Debug, Encode, Decode)]
#[cbor(transparent)]
pub struct Refuse<'a>(pub RefuseReason<'a>);

impl Message for Refuse<'static> {
    const SIZE_LIMIT: usize = 5760;
    const TAG: u8 = 2;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Done;
}

#[derive(Debug, Encode, Decode)]
#[cbor(transparent)]
pub struct QueryReply<VD>(pub VersionTable<VD>);

impl<D> Message for QueryReply<D>
where
    D: Encode<()> + for<'a> Decode<'a, ()> + 'static,
{
    const SIZE_LIMIT: usize = 5760;
    const TAG: u8 = 3;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Done;
}

#[derive(Debug)]
pub struct VersionTable<D> {
    pub versions: Vec<(Version, D)>,
}

impl<C, D> Encode<C> for VersionTable<D>
where
    D: Encode<C>,
{
    fn encode<W: encode::Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        e.map(self.versions.len() as u64)?;
        for (version, data) in &self.versions {
            e.u16(*version)?;
            e.encode_with(data, ctx)?;
        }
        Ok(())
    }
}

impl<'a, C, D> Decode<'a, C> for VersionTable<D>
where
    D: Decode<'a, C>,
{
    fn decode(d: &mut minicbor::Decoder<'a>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        let versions: Vec<(u16, D)> = d.map_iter_with(ctx)?.collect::<Result<_, _>>()?;
        Ok(Self { versions })
    }
}

#[derive(Debug, Encode, Decode)]
pub struct NodeToNodeVersionData {
    #[n(0)]
    pub network_magic: NetworkMagic,
    #[n(1)]
    pub diffusion_mode: bool,
    #[n(2)]
    #[cbor(with = "cbor_util::bool_as_u8")]
    pub peer_sharing: bool,
    #[n(3)]
    pub query: bool,
}

#[derive(Debug, Encode, Decode)]
pub struct NodeToClientVersionData {
    #[n(0)]
    pub network_magic: NetworkMagic,
    #[n(1)]
    pub query: bool,
}

#[derive(Debug, Encode, Decode)]
#[cbor(flat)]
pub enum RefuseReason<'a> {
    #[n(0)]
    VersionMismatch(#[n(0)] Vec<Version>),
    #[n(1)]
    HandshakeDecodeError(#[n(0)] Version, #[n(1)] Cow<'a, str>),
    #[n(2)]
    Refused(#[n(0)] Version, #[n(1)] Cow<'a, str>),
}

pub type Version = u16;
