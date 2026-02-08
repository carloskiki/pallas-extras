//! Host of [`SingleUse`] and [`VerifyingKey`].

use crate::{Evolve, KeyEvolvingSignature};
use digest::{
    Key,
    common::{Generate, KeySizeUser, TryKeyInit},
};
use ref_cast::RefCast;
use signature::{Keypair, KeypairRef, Signer, Verifier};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

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

impl<T> TryKeyInit for SingleUse<T>
where
    T: TryKeyInit,
{
    fn new(key: &Key<Self>) -> Result<Self, digest::common::InvalidKey> {
        T::new(key).map(SingleUse)
    }
}

impl<T> Generate for SingleUse<T>
where
    T: Generate,
{
    fn try_generate_from_rng<R: digest::rand_core::TryCryptoRng + ?Sized>(
        rng: &mut R,
    ) -> Result<Self, R::Error> {
        T::try_generate_from_rng(rng).map(SingleUse)
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

impl<S, T> Verifier<KeyEvolvingSignature<&S>> for SingleUse<T>
where
    T: KeypairRef,
    T::VerifyingKey: Verifier<S>,
{
    fn verify(
        &self,
        msg: &[u8],
        signature: &KeyEvolvingSignature<&S>,
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

/// This is a transparent wrapper around `VK`.
///
/// Importantly, it implements [`Verifier<KeyEvolvingSignature<S>>`] if
/// [`VK::VerifyingKey`](KeypairRef::VerifyingKey) implements [`Verifier<S>`].
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Immutable,
    Unaligned,
    IntoBytes,
    FromBytes,
    KnownLayout,
    ref_cast::RefCast,
)]
#[repr(transparent)]
pub struct VerifyingKey<VK>(pub VK);

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

impl<S, VK: Verifier<S>> Verifier<KeyEvolvingSignature<&S>> for VerifyingKey<VK> {
    fn verify(
        &self,
        msg: &[u8],
        KeyEvolvingSignature {
            signature: s,
            period,
        }: &KeyEvolvingSignature<&S>,
    ) -> Result<(), signature::Error> {
        if *period == 0 {
            self.0.verify(msg, s)
        } else {
            Err(signature::Error::new())
        }
    }
}

impl<VK: KeySizeUser> KeySizeUser for VerifyingKey<VK> {
    type KeySize = VK::KeySize;
}

#[cfg(test)]
mod tests {
    use super::SingleUse;
    use crate::Evolve;
    use digest::crypto_common::Generate;
    use ed25519_dalek::SigningKey;

    #[test]
    fn cannot_evolve() {
        let key = SingleUse::<SigningKey>::generate();
        assert!(key.evolve().is_none());
    }
}
