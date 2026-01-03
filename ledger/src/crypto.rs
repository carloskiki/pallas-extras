use digest::consts::{U28, U32};

pub type Blake2b224 = blake2::Blake2b<U28>;
pub type Blake2b224Digest = [u8; 28];
pub type Blake2b256 = blake2::Blake2b<U32>;
pub type Blake2b256Digest = [u8; 32];

pub type VerifyingKey = [u8; 32];
pub type Signature = ed25519_dalek::Signature;
pub type ExtendedVerifyingKey = bip32::ExtendedVerifyingKey;

pub mod kes {
    pub type VerifyingKey = kes::sum::VerifyingKey<super::Blake2b256>;
    pub type SigningKey = kes::sum::Pow6<kes::SingleUse<ed25519_dalek::SigningKey>, super::Blake2b256>;
    pub type Signature = kes::sum::Pow6Signature<ed25519_dalek::Signature, kes::SingleUse<ed25519_dalek::SigningKey>, super::Blake2b256>;
}
