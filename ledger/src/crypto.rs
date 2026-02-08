use digest::{consts::{U28, U32}, common::KeySizeUser};

pub type Blake2b224 = blake2::Blake2b<U28>;
pub type Blake2b224Digest = [u8; 28];
pub type Blake2b256 = blake2::Blake2b<U32>;
pub type Blake2b256Digest = [u8; 32];

pub type VerifyingKey = ed25519_dalek::pkcs8::PublicKeyBytes;
pub type Signature = ed25519_dalek::Signature;
pub type ExtendedVerifyingKey = bip32::ExtendedVerifyingKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keypair {
    pub secret: ed25519_dalek::SecretKey,
    pub public: VerifyingKey,
}

impl KeySizeUser for Keypair {
    type KeySize = U32;
}

impl AsRef<VerifyingKey> for Keypair {
    fn as_ref(&self) -> &VerifyingKey {
        &self.public
    }
}

impl ed25519::signature::KeypairRef for Keypair {
    type VerifyingKey = VerifyingKey;
}

pub mod kes {

    pub type VerifyingKey = kes::sum::VerifyingKey<super::Blake2b256>;
    pub type Signature = kes::sum::Pow6Signature<
        ed25519_dalek::Signature,
        kes::SingleUse<super::Keypair>,
        super::Blake2b256,
    >;

}
