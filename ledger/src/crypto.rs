//! Cryptographic primitives

use digest::{
    common::KeySizeUser,
    consts::{U28, U32},
};

pub(crate) type Blake2b224 = blake2::Blake2b<U28>;
type Blake2b256 = blake2::Blake2b<U32>;
/// Blake2b224 hash value.
pub type Blake2b224Digest = [u8; 28];
/// Blake2b256 hash value.
pub type Blake2b256Digest = [u8; 32];

pub type VerifyingKey = ed25519_dalek::pkcs8::PublicKeyBytes;
pub type Signature = ed25519_dalek::Signature;
pub type ExtendedVerifyingKey = bip32::ExtendedVerifyingKey;

/// Pair of serialized secret and verifying keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keypair {
    pub secret: ed25519_dalek::SecretKey,
    pub verifying: VerifyingKey,
}

impl KeySizeUser for Keypair {
    type KeySize = U32;
}

impl AsRef<VerifyingKey> for Keypair {
    fn as_ref(&self) -> &VerifyingKey {
        &self.verifying
    }
}

impl ed25519::signature::KeypairRef for Keypair {
    type VerifyingKey = VerifyingKey;
}

pub mod kes {
    //! Key evolving cryptographic primitives.
    
    pub type VerifyingKey = kes::sum::VerifyingKey<super::Blake2b256>;
    #[allow(private_interfaces)]
    pub type Signature = kes::sum::Pow6Signature<
        ed25519_dalek::Signature,
        kes::SingleUse<super::Keypair>,
        super::Blake2b256,
    >;
}
