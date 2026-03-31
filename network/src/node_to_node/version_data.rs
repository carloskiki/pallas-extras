use crate::NetworkMagic;
use tinycbor::{CborLen, Decode, Encode};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Encode, Decode, CborLen)]
pub struct VersionData {
    pub network_magic: NetworkMagic,
    pub diffusion_mode: bool,
    #[cbor(with = "BoolU")]
    pub peer_sharing: bool,
    pub query: bool,
}

#[repr(transparent)]
struct BoolU(bool);

impl From<BoolU> for bool {
    fn from(value: BoolU) -> Self {
        value.0
    }
}

impl<'a> From<&'a bool> for &'a BoolU {
    fn from(value: &bool) -> Self {
        // Safety: `BoolU` is a transparent wrapper around `bool`.
        unsafe { &*(value as *const bool as *const BoolU) }
    }
}

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
