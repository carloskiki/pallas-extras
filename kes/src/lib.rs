use signature::Signer;

pub mod single_use;
pub mod sum;

pub use single_use::SingleUse;
pub use sum::Sum;

/// Trait for forward secure key evolution.
pub trait Evolve: Sized {
    /// The number of periods for the key.
    ///
    /// Can be seen the number of times the key can evolve plus 1.
    const PERIOD_COUNT: u32;

    /// Evolve the key to the next period.
    fn evolve(self) -> Option<Self>;

    /// Every time the key evolves, the period is incremented by 1, starting at 0.
    fn period(&self) -> u32;

    /// Sign a message and then evolve the key.
    fn try_sign_evolve<S>(self, msg: &[u8]) -> signature::Result<(Option<Self>, S)>
    where
        Self: Signer<S>,
    {
        let signature = self.try_sign(msg)?;
        let evolution = self.evolve();
        Ok((evolution, signature))
    }
}

/// Also know as KES.
///
/// A signature with a period.
#[derive(Debug)]
pub struct KeyEvolvingSignature<'a, S> {
    /// The signature.
    pub signature: &'a S,
    /// The period.
    pub period: u32,
}

impl<S> Clone for KeyEvolvingSignature<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for KeyEvolvingSignature<'_, S> {}

#[cfg(test)]
pub(crate) mod tests {
    use crate::single_use::SingleUse;
    use blake2::Blake2b;
    use digest::{array::Array, consts::U32, crypto_common::KeySizeUser};
    use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
    use signature::{KeypairRef, Signer, Verifier};

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct SkWrapper(pub SigningKey);

    impl AsRef<VerifyingKey> for SkWrapper {
        fn as_ref(&self) -> &VerifyingKey {
            self.0.as_ref()
        }
    }

    impl KeypairRef for SkWrapper {
        type VerifyingKey = <SigningKey as KeypairRef>::VerifyingKey;
    }

    impl KeySizeUser for SkWrapper {
        type KeySize = U32;
    }

    impl From<[u8; 32]> for SkWrapper {
        fn from(bytes: [u8; 32]) -> Self {
            SkWrapper(SigningKey::from_bytes(&bytes))
        }
    }

    impl From<Array<u8, U32>> for SkWrapper {
        fn from(bytes: Array<u8, U32>) -> Self {
            SkWrapper(SigningKey::from_bytes(bytes.as_ref()))
        }
    }

    impl Signer<Signature> for SkWrapper {
        fn try_sign(&self, msg: &[u8]) -> Result<Signature, signature::Error> {
            self.0.try_sign(msg)
        }
    }

    impl Verifier<Signature> for SkWrapper {
        fn verify(&self, msg: &[u8], signature: &Signature) -> Result<(), signature::Error> {
            self.0.verify(msg, signature)
        }
    }

    pub type KeyBase = SingleUse<SkWrapper>;
    pub type THash = Blake2b<U32>;
}
