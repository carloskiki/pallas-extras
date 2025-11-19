use digest::{Digest, OutputSizeUser, crypto_common::KeySizeUser};
use either::Either::{Left, Right};
use signature::{Keypair, KeypairRef, Signer, Verifier};

use crate::{Evolve, KeyEvolvingSignature, sum};

/// Similar to [`sum::Signature`], but with a compact representation.
///
/// This has a smaller size than [`sum::Signature`], at the cost of slower verification. This signature
/// type can only be used when both the left and right parts of the sum are the same type.
///
/// This can be chosen as the output of [`Sum::sign`](sum::Sum) by setting the `Signer` type parameter.
/// To [`Verifier::verify`] this signature, it must first be assembled into a
/// [`KeyEvolvingSignature`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompactSignature<S, KP>
where
    KP: KeypairRef,
{
    pub signature: S,
    pub vkey: KP::VerifyingKey,
}

impl<S, KP, H> Signer<CompactSignature<S, sum::Double<KP, H>>> for sum::Pow2<KP, H>
where
    KP: KeypairRef + KeySizeUser,
    H: OutputSizeUser,
    sum::Double<KP, H>: Signer<S>,
{
    fn try_sign(
        &self,
        msg: &[u8],
    ) -> Result<CompactSignature<S, sum::Double<KP, H>>, signature::Error> {
        match &self.inner {
            Left((left, right_vkey)) => Ok(CompactSignature {
                signature: left.try_sign(msg)?,
                vkey: right_vkey.clone(),
            }),
            Right((right, left_vkey)) => Ok(CompactSignature {
                signature: right.try_sign(msg)?,
                vkey: left_vkey.clone(),
            }),
        }
    }
}

impl<'a, S, KP, H> Verifier<KeyEvolvingSignature<'a, CompactSignature<S, KP>>>
    for sum::VerifyingKey<H>
where
    KP: KeypairRef + KeySizeUser + Evolve,
    KP::VerifyingKey: InnerVerifier<KeyEvolvingSignature<'a, S>>
        + FromSignature<KeyEvolvingSignature<'a, S>>
        + AsRef<[u8]>,
    H: Digest,
{
    fn verify(
        &self,
        msg: &[u8],
        signature: &KeyEvolvingSignature<'a, CompactSignature<S, KP>>,
    ) -> Result<(), signature::Error> {
        let vkey = Self::from_signature(*signature);
        if &vkey != self {
            return Err(signature::Error::new());
        }

        sum::VerifyingKey::<H>::verify_inner(msg, signature)
    }
}

impl<'a, S, KP, H> Verifier<KeyEvolvingSignature<'a, CompactSignature<S, KP>>>
    for sum::Double<KP, H>
where
    KP: KeypairRef + KeySizeUser + Evolve,
    KP::VerifyingKey: InnerVerifier<KeyEvolvingSignature<'a, S>>
        + FromSignature<KeyEvolvingSignature<'a, S>>
        + AsRef<[u8]>,
    H: Digest,
{
    fn verify(
        &self,
        msg: &[u8],
        signature: &KeyEvolvingSignature<'a, CompactSignature<S, KP>>,
    ) -> Result<(), signature::Error> {
        self.verifying_key().verify(msg, signature)
    }
}

pub type Pow2CompactSignature<S, KP, H> =
    CompactSignature<sum::Signature<S, KP, KP>, sum::Double<KP, H>>;
pub type Pow3CompactSignature<S, KP, H> =
    CompactSignature<Pow2CompactSignature<S, KP, H>, sum::Pow2<KP, H>>;
pub type Pow4CompactSignature<S, KP, H> =
    CompactSignature<Pow3CompactSignature<S, KP, H>, sum::Pow3<KP, H>>;
pub type Pow5CompactSignature<S, KP, H> =
    CompactSignature<Pow4CompactSignature<S, KP, H>, sum::Pow4<KP, H>>;
pub type Pow6CompactSignature<S, KP, H> =
    CompactSignature<Pow5CompactSignature<S, KP, H>, sum::Pow5<KP, H>>;

trait FromSignature<S> {
    fn from_signature(signature: S) -> Self;
}

impl<'a, S, KP, H> FromSignature<KeyEvolvingSignature<'a, CompactSignature<S, KP>>>
    for sum::VerifyingKey<H>
where
    KP: KeypairRef + Evolve,
    KP::VerifyingKey: FromSignature<KeyEvolvingSignature<'a, S>> + AsRef<[u8]>,
    H: Digest,
{
    fn from_signature(
        KeyEvolvingSignature {
            signature: CompactSignature { vkey, signature },
            mut period,
        }: KeyEvolvingSignature<'a, CompactSignature<S, KP>>,
    ) -> Self {
        let left_side = period < KP::PERIOD_COUNT;
        period -= (!left_side as u32) * KP::PERIOD_COUNT;
        let kes = KeyEvolvingSignature { signature, period };
        let recomputed_vkey = KP::VerifyingKey::from_signature(kes);

        let mut hasher = H::new();

        if left_side {
            hasher.update(recomputed_vkey);
            hasher.update(vkey.as_ref());
        } else {
            hasher.update(vkey.as_ref());
            hasher.update(recomputed_vkey);
        }

        sum::VerifyingKey(hasher.finalize())
    }
}

impl<'a, S, KP, H> FromSignature<KeyEvolvingSignature<'a, sum::Signature<S, KP, KP>>>
    for sum::VerifyingKey<H>
where
    KP: KeypairRef<VerifyingKey: AsRef<[u8]>>,
    H: Digest,
{
    fn from_signature(
        KeyEvolvingSignature {
            signature:
                sum::Signature {
                    left_vkey,
                    right_vkey,
                    ..
                },
            ..
        }: KeyEvolvingSignature<'a, sum::Signature<S, KP, KP>>,
    ) -> Self {
        let mut hasher = H::new();
        hasher.update(left_vkey.as_ref());
        hasher.update(right_vkey.as_ref());
        sum::VerifyingKey(hasher.finalize())
    }
}

trait InnerVerifier<S> {
    fn verify_inner(msg: &[u8], signature: &S) -> Result<(), signature::Error>;
}

impl<'a, S, KP, H> InnerVerifier<KeyEvolvingSignature<'a, CompactSignature<S, KP>>>
    for sum::VerifyingKey<H>
where
    KP: KeypairRef + KeySizeUser + Evolve,
    KP::VerifyingKey: InnerVerifier<KeyEvolvingSignature<'a, S>>,
    H: Digest,
{
    fn verify_inner(
        msg: &[u8],
        signature: &KeyEvolvingSignature<'a, CompactSignature<S, KP>>,
    ) -> Result<(), signature::Error> {
        let kes = KeyEvolvingSignature {
            signature: &signature.signature.signature,
            period: signature
                .period
                .checked_sub(KP::PERIOD_COUNT)
                .unwrap_or(signature.period),
        };
        KP::VerifyingKey::verify_inner(msg, &kes)
    }
}

impl<'a, S, KP, H> InnerVerifier<KeyEvolvingSignature<'a, sum::Signature<S, KP, KP>>>
    for sum::VerifyingKey<H>
where
    KP: KeypairRef + KeySizeUser + Evolve,
    KP::VerifyingKey: Verifier<KeyEvolvingSignature<'a, S>>,
    H: Digest,
{
    fn verify_inner(
        msg: &[u8],
        KeyEvolvingSignature { signature, period }: &KeyEvolvingSignature<
            'a,
            sum::Signature<S, KP, KP>,
        >,
    ) -> Result<(), signature::Error> {
        let mut period = *period;
        let vkey = if period < KP::PERIOD_COUNT {
            &signature.left_vkey
        } else {
            period -= KP::PERIOD_COUNT;
            &signature.right_vkey
        };
        let kes = KeyEvolvingSignature {
            signature: &signature.signature,
            period,
        };
        vkey.verify(msg, &kes)
    }
}

#[cfg(test)]
mod tests {
    use digest::array::Array;
    use rand::random;
    use signature::{Keypair, Signer};

    use super::*;
    use crate::{
        Evolve, sum,
        tests::{KeyBase, THash},
    };

    const MESSAGES: [&[u8]; 8] = [
        b"foo",
        b"bar",
        b"baz",
        b"qux",
        b"quux",
        b"123456",
        b"abcdef",
        b"Hello, world!",
    ];

    #[test]
    fn verifying_key_from_signature() {
        let seed: [u8; 32] = random();
        let msg = b"Hello, world!";

        let key: sum::Pow2<KeyBase, THash> = sum::Pow2::from(Array::from(seed));
        let signature: Pow2CompactSignature<_, _, _> = key.sign(msg);
        let kes = KeyEvolvingSignature {
            signature: &signature,
            period: key.period(),
        };
        let vkey = sum::VerifyingKey::<THash>::from_signature(kes);
        assert_eq!(vkey, key.verifying_key());

        let key: sum::Pow6<KeyBase, THash> = sum::Pow6::from(Array::from(seed));
        let signature: Pow6CompactSignature<_, _, _> = key.sign(msg);
        let kes = KeyEvolvingSignature {
            signature: &signature,
            period: key.period(),
        };
        let vkey = sum::VerifyingKey::<THash>::from_signature(kes);
        assert_eq!(vkey, key.verifying_key());
    }

    #[test]
    fn verify() {
        let seed: [u8; 32] = random();
        let msg = b"Hello, world!";

        let key: sum::Pow3<KeyBase, THash> = sum::Pow3::from(Array::from(seed));
        let signature: Pow3CompactSignature<_, _, _> = key.sign(msg);
        let kes = KeyEvolvingSignature {
            signature: &signature,
            period: key.period(),
        };
        assert!(key.verifying_key().verify(msg, &kes).is_ok());
    }

    #[test]
    fn can_verify_from_all_evolutions() {
        let key: [u8; 32] = random();
        let mut pow6: sum::Pow6<KeyBase, THash> = sum::Pow6::from(Array::from(key));

        let vkey = pow6.verifying_key();
        let raw_signature: Pow6CompactSignature<_, _, _> = pow6.try_sign(MESSAGES[0]).unwrap();
        let mut signature = KeyEvolvingSignature {
            signature: &raw_signature,
            period: pow6.period(),
        };
        assert!(vkey.verify(MESSAGES[0], &signature).is_ok());

        for i in 1..sum::Pow6::<KeyBase, THash>::PERIOD_COUNT {
            pow6 = pow6.evolve().unwrap();
            let index = i as usize % MESSAGES.len();
            let new_raw_signature = pow6.try_sign(MESSAGES[index]).unwrap();
            signature = KeyEvolvingSignature {
                signature: &new_raw_signature,
                period: pow6.period(),
            };
            assert!(vkey.verify(MESSAGES[index], &signature).is_ok());
        }
    }

    #[test]
    fn different_vkey_verification_fails() {
        let key: [u8; 32] = random();
        let mut kes1: sum::Pow6<KeyBase, THash> = sum::Pow6::from(Array::from(key));
        let other_key: [u8; 32] = random();
        let other_vkey =
            sum::Pow6::<KeyBase, THash>::from(
                Array::from(other_key)
            ).verifying_key();

        for msg in MESSAGES {
            let raw_signature: Pow6CompactSignature<_, _, _> = kes1.try_sign(msg).unwrap();
            let signature = KeyEvolvingSignature {
                signature: &raw_signature,
                period: kes1.period(),
            };

            assert!(other_vkey.verify(msg, &signature).is_err());
            kes1 = kes1.evolve().unwrap();
        }
    }

    #[test]
    fn wrong_signature_period_fails() {
        let key: [u8; 32] = random();
        let mut skey: sum::Pow6<KeyBase, THash> = sum::Pow6::from(Array::from(key));

        for msg in MESSAGES {
            let signature: Pow6CompactSignature<_, _, _> = skey.try_sign(msg).unwrap();
            let mut kes = KeyEvolvingSignature {
                signature: &signature,
                period: skey.period(),
            };
            for x in 0..100 {
                kes.period = x;
                assert!(skey.verify(msg, &kes).is_ok() == (x == skey.period()));
            }
            skey = skey.evolve().unwrap();
        }
    }
}
