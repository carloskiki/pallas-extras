use bip32::curve25519_dalek::edwards::CompressedEdwardsY;
use tinycbor::{
    CborLen, Decode, Decoder, Encode, Encoder, Write,
    collections::{self, fixed},
};
use tinycbor_derive::{CborLen, Decode, Encode};
use zerocopy::transmute;

use crate::crypto::VerifyingKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Data {
    #[n(0)]
    VerifyingKey(#[cbor(with = "ExtendedVerifyingKey")] bip32::ExtendedVerifyingKey),
    #[n(1)]
    Redeem(VerifyingKey),
}

#[repr(transparent)]
struct ExtendedVerifyingKey(bip32::ExtendedVerifyingKey);

impl From<ExtendedVerifyingKey> for bip32::ExtendedVerifyingKey {
    fn from(pk: ExtendedVerifyingKey) -> Self {
        pk.0
    }
}

impl From<&bip32::ExtendedVerifyingKey> for &ExtendedVerifyingKey {
    fn from(pk: &bip32::ExtendedVerifyingKey) -> Self {
        // SAFETY: PublicKey is #[repr(transparent)] over ExtendedVerifyingKey
        unsafe { &*(pk as *const bip32::ExtendedVerifyingKey as *const ExtendedVerifyingKey) }
    }
}

impl Encode for ExtendedVerifyingKey {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        // CBOR bytestring len 64 header
        e.0.write_all(&[0x58, 0x40])?;
        e.0.write_all(&self.0.key.compress().0)?;
        e.0.write_all(&self.0.chain_code)
    }
}

impl Decode<'_> for ExtendedVerifyingKey {
    type Error = fixed::Error<bip32::InvalidKey>;

    fn decode(d: &mut Decoder<'_>) -> Result<Self, Self::Error> {
        let bytes: [u8; 64] =
            Decode::decode(d).map_err(|e: fixed::Error<_>| e.map(|e| match e {}))?;
        let [key, chain_code]: [[u8; 32]; 2] = transmute!(bytes);
        let key = CompressedEdwardsY(key)
            .decompress()
            .ok_or(collections::Error::Element(bip32::InvalidKey))?;
        Ok(Self(bip32::ExtendedVerifyingKey { key, chain_code }))
    }
}

impl CborLen for ExtendedVerifyingKey {
    fn cbor_len(&self) -> usize {
        64.cbor_len() + 64
    }
}
