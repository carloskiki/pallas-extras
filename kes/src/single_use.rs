//! Host of [`SingleUse`] and [`VerifyingKey`].

use digest::{KeyInit, crypto_common::KeySizeUser};
use ref_cast::RefCast;
use signature::{Keypair, KeypairRef, Signer, Verifier};

use crate::{Evolve, KeyEvolvingSignature};

/// A implementation of [`Evolve`] that returns [`None`] when [`Evolve::evolve`] is called.
///
/// In other words, it has `1` period and cannot evolve. This is useful to implement the
/// composition structures defined by MMM.
///
/// This implements [`Signer`], [`Verifier`] for [`KeyEvolvingSignature<S>`] where `S` is the
/// signature type for `T`. This also implements [`KeypairRef`] by returning
/// [`VerifyingKey`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SingleUse<T>(pub T);

impl<T: KeySizeUser> KeySizeUser for SingleUse<T> {
    type KeySize = T::KeySize;
}

impl<T: KeyInit> KeyInit for SingleUse<T> {
    fn new(key: &digest::Key<Self>) -> Self {
        SingleUse(T::new(key))
    }
}

impl<T> Evolve for SingleUse<T> {
    const PERIOD_COUNT: u32 = 1;

    fn period(&self) -> u32 {
        0
    }

    fn evolve(self) -> Option<Self> {
        None
    }
}

impl<S, T> Signer<S> for SingleUse<T>
where
    T: Signer<S>,
{
    fn try_sign(&self, msg: &[u8]) -> Result<S, signature::Error> {
        self.0.try_sign(msg)
    }
}

impl<S, T> Verifier<KeyEvolvingSignature<'_, S>> for SingleUse<T>
where
    T: KeypairRef,
    T::VerifyingKey: Verifier<S>,
{
    fn verify(
        &self,
        msg: &[u8],
        signature: &KeyEvolvingSignature<S>,
    ) -> Result<(), signature::Error> {
        self.verifying_key().verify(msg, signature)
    }
}

impl<T: KeypairRef> KeypairRef for SingleUse<T> {
    type VerifyingKey = VerifyingKey<T::VerifyingKey>;
}

impl<T: KeypairRef> AsRef<VerifyingKey<T::VerifyingKey>> for SingleUse<T> {
    fn as_ref(&self) -> &VerifyingKey<T::VerifyingKey> {
        VerifyingKey::ref_cast(self.0.as_ref())
    }
}

// A newtype wrapper is needed because we need for
/// This is a transparent wrapper around `VK`.
///
/// Importantly, it implements [`Verifier<KeyEvolvingSignature<S>>`] if
/// [`VK::VerifyingKey`](KeypairRef::VerifyingKey) implements [`Verifier<S>`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, RefCast)]
#[repr(transparent)]
pub struct VerifyingKey<VK>(VK);

impl<'a, VK: TryFrom<&'a [u8]>> TryFrom<&'a [u8]> for VerifyingKey<VK> {
    type Error = VK::Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        VK::try_from(value).map(VerifyingKey)
    }
}

impl<VK: AsRef<[u8]>> AsRef<[u8]> for VerifyingKey<VK> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<S, VK: Verifier<S>> Verifier<KeyEvolvingSignature<'_, S>> for VerifyingKey<VK> {
    fn verify(
        &self,
        msg: &[u8],
        KeyEvolvingSignature {
            signature: s,
            period,
        }: &KeyEvolvingSignature<S>,
    ) -> Result<(), signature::Error> {
        (*period == 0)
            .then(|| self.0.verify(msg, s))
            .unwrap_or(Err(signature::Error::new()))
    }
}

impl<VK: KeySizeUser> KeySizeUser for VerifyingKey<VK> {
    type KeySize = VK::KeySize;
}

#[cfg(test)]
mod tests {
    use crate::{Evolve as _, tests::SkWrapper};
    use digest::KeyInit;

    use super::SingleUse;

    #[test]
    fn cannot_evolve() {
        let seed: [u8; 32] = [0; 32];
        let key: SingleUse<SkWrapper> = SingleUse::new(&seed.into());
        assert!(key.evolve().is_none());
    }
}
