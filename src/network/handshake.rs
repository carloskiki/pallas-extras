use minicbor::{Decode, Encode, Encoder, encode};

use super::NetworkMagic;

pub type Version = u16;

#[derive(Debug, Encode, Decode)]
#[cbor(flat)]
pub enum ClientMessage<D> {
    #[n(0)]
    ProposeVersions(#[n(0)] VersionTable<D>),
}

#[derive(Debug, Encode, Decode)]
#[cbor(flat)]
pub enum ServerMessage<'a, D> {
    #[n(1)]
    AcceptVersion(#[n(0)] Version, #[n(1)] D),
    #[n(2)]
    Refuse(#[b(0)] RefuseReason<'a>),
    #[n(3)]
    QueryReply(#[n(0)] VersionTable<D>),
}

#[derive(Debug)]
pub struct VersionTable<D> {
    pub versions: Vec<(Version, D)>,
}

impl<C, D> Encode<C> for VersionTable<D>
where D: Encode<C>
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
where D: Decode<'a, C>
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
    #[cbor(with = "crate::cbor::bool_as_u8")]
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
    HandshakeDecodeError(#[n(0)] Version, #[b(1)] &'a str),
    #[n(2)]
    Refused(#[n(0)] Version, #[b(1)] &'a str),
}
