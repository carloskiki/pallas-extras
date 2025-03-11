use minicbor::{Decode, Encode, Encoder, encode};

use super::NetworkMagic;

pub type Version = u16;

#[derive(Debug, Encode, Decode)]
#[cbor(flat)]
pub enum ClientMessage {
    #[n(0)]
    ProposeVersions(#[n(0)] VersionTable),
}

#[derive(Debug, Encode, Decode)]
#[cbor(flat)]
pub enum ServerMessage<'a> {
    #[n(1)]
    AcceptVersion(#[n(0)] Version, #[n(1)] VersionData),
    #[n(2)]
    Refuse(#[b(0)] RefuseReason<'a>),
    #[n(3)]
    QueryReply(#[n(0)] VersionTable),
}

#[derive(Debug)]
pub struct VersionTable {
    pub versions: Vec<(Version, VersionData)>,
}

impl<C> Encode<C> for VersionTable {
    fn encode<W: encode::Write>(
        &self,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        e.map(self.versions.len() as u64)?;
        for (version, data) in &self.versions {
            e.u16(*version)?;
            e.encode(data)?;
        }
        Ok(())
    }
}

impl<C> Decode<'_, C> for VersionTable {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let versions: Vec<(u16, VersionData)> = d.map_iter()?.collect::<Result<_, _>>()?;
        Ok(Self { versions })
    }
}

#[derive(Debug, Encode, Decode)]
pub struct VersionData {
    #[n(0)]
    pub network_magic: NetworkMagic,
    #[n(1)]
    pub diffusion_mode: bool,
    #[n(2)]
    #[cbor(with = "bool_as_u8")]
    pub peer_sharing: bool,
    #[n(3)]
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

mod bool_as_u8 {
    use minicbor::{Decoder, Encoder, decode as de, encode as en};

    pub fn encode<C, W: en::Write>(
        value: &bool,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        e.u8(*value as u8)?.ok()
    }

    pub fn decode<Ctx>(d: &mut Decoder<'_>, _: &mut Ctx) -> Result<bool, de::Error> {
        d.u8().map(|v| v != 0)
    }
}
