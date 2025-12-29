use bip32::{ExtendedVerifyingKey, curve25519_dalek::edwards::CompressedEdwardsY};
use tinycbor::{CborLen, Decode, Decoder, Encode, Encoder, Write};
use tinycbor_derive::{CborLen, Decode, Encode};
use zerocopy::transmute;

use crate::crypto::VerifyingKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, CborLen)]
pub enum Data {
    #[n(0)]
    PublicKey(ExtendedVerifyingKey),
    #[n(1)]
    Redeem(#[cbor(with = "PublicKey")] VerifyingKey),
}

#[repr(transparent)]
struct PublicKey(ExtendedVerifyingKey);

impl From<PublicKey> for ExtendedVerifyingKey {
    fn from(pk: PublicKey) -> Self {
        pk.0
    }
}

impl From<&ExtendedVerifyingKey> for &PublicKey {
    fn from(pk: &ExtendedVerifyingKey) -> Self {
        // SAFETY: PublicKey is #[repr(transparent)] over ExtendedVerifyingKey
        unsafe { &*(pk as *const ExtendedVerifyingKey as *const PublicKey) }
    }
}

impl Encode for PublicKey {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        // CBOR bytestring len 64 header
        e.0.write_all(&[0x58, 0x40])?;
        e.0.write_all(self.0.key.compress())?;
        e.0.write_all(&self.0.chain_code)?;
    }
}

impl Decode<'_> for PublicKey {
    type Error = tinycbor::collections::Error<bip32::InvalidKey>;

    fn decode(d: &mut Decoder<'_>) -> Result<Self, Self::Error> {
        let bytes: [u8; 64] = Decode::decode(d)?;
        let [key, chain_code]: [[u8; 32]; 2] = transmute!(bytes);
        let key = CompressedEdwardsY(key)
            .decompress()
            .ok_or(bip32::InvalidKey)?;
        Ok(ExtendedVerifyingKey { key, chain_code })
    }
}

impl CborLen for PublicKey {
    fn cbor_len(&self) -> usize {
        64.cbor_len() + 64
    }
}
