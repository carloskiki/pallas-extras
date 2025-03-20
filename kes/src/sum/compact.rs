use digest::{crypto_common::KeySizeUser, Digest, KeyInit, OutputSizeUser};
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

trait ToVerifyingKey<H: OutputSizeUser> {
    fn to_verifying_key(self) -> sum::VerifyingKey<H>;
}

#[doc(hidden)]
impl<'a, S, KP, H> ToVerifyingKey<H>
    for KeyEvolvingSignature<'a, CompactSignature<CompactSignature<S, KP>, KP>>
where
    KP: KeypairRef<VerifyingKey: AsRef<[u8]>> + Evolve,
    H: Digest,
{
    fn to_verifying_key(
        self
    ) -> sum::VerifyingKey<H> {
        let KeyEvolvingSignature {
            signature:
                CompactSignature {
                    signature:
                        CompactSignature {
                            vkey: inner_vkey, ..
                        },
                    vkey: outer_vkey,
                },
            period,
        } = self;
        let mut hasher = H::new();

        if period < KP::PERIOD_COUNT {
            hasher.update(inner_vkey);
            hasher.update(outer_vkey);
        } else {
            hasher.update(outer_vkey);
            hasher.update(inner_vkey);
        }

        sum::VerifyingKey(hasher.finalize())
    }
}

#[doc(hidden)]
impl<'a, S, KP, H>
    ToVerifyingKey<H>
    for KeyEvolvingSignature<'a, CompactSignature<CompactSignature<S, KP>, sum::Double<KP, H>>>
where
    KP: KeypairRef<VerifyingKey: AsRef<[u8]>> + KeyInit + Evolve,
    H: Digest,
    KeyEvolvingSignature<'a, CompactSignature<S, KP>>: ToVerifyingKey<H>,
{
    fn to_verifying_key(
        self
    ) -> sum::VerifyingKey<H> {
        let KeyEvolvingSignature {
            signature:
                CompactSignature {
                    signature,
                    vkey: other_vkey,
                },
            period,
        } = self;

        
        let mut hasher = H::new();
        let mut kes = KeyEvolvingSignature { signature, period };
        let left_side = period < sum::Double::<KP, H>::PERIOD_COUNT;
        if !left_side {
            kes.period -= sum::Double::<KP, H>::PERIOD_COUNT;
        }
        
        let vkey = kes.to_verifying_key();

        if left_side {
            hasher.update(vkey);
            hasher.update(other_vkey);
        } else {
            hasher.update(other_vkey);
            hasher.update(vkey);
        }

        sum::VerifyingKey(hasher.finalize())
    }
}

impl<S, KP, H> Signer<CompactSignature<CompactSignature<S, KP>, KP>> for sum::Double<KP, H>
where
    KP: KeypairRef + KeySizeUser + Signer<S>,
    H: OutputSizeUser,
{
    fn try_sign(
        &self,
        msg: &[u8],
    ) -> Result<CompactSignature<CompactSignature<S, KP>, KP>, signature::Error> {
        match &self.inner {
            Left((left, right_vkey)) => Ok(CompactSignature {
                signature: CompactSignature {
                    signature: left.try_sign(msg)?,
                    vkey: left.verifying_key(),
                },
                vkey: right_vkey.clone(),
            }),
            Right((right, left_vkey)) => Ok(CompactSignature {
                signature: CompactSignature {
                    signature: right.try_sign(msg)?,
                    vkey: right.verifying_key(),
                },
                vkey: left_vkey.clone(),
            }),
        }
    }
}

impl<S, KP, H> Signer<CompactSignature<CompactSignature<S, KP>, sum::Double<KP, H>>>
    for sum::Pow2<KP, H>
where
    KP: KeypairRef + KeySizeUser,
    H: OutputSizeUser,
    sum::Double<KP, H>: Signer<CompactSignature<S, KP>>,
{
    fn try_sign(
        &self,
        msg: &[u8],
    ) -> Result<CompactSignature<CompactSignature<S, KP>, sum::Double<KP, H>>, signature::Error>
    {
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

impl<'a, S, KP, H> Verifier<KeyEvolvingSignature<'a, CompactSignature<CompactSignature<S, KP>, KP>>>
    for sum::VerifyingKey<H>
where
    KP: KeypairRef + KeySizeUser + Evolve,
    KP::VerifyingKey: AsRef<[u8]> + Verifier<KeyEvolvingSignature<'a, S>>,
    H: Digest,
{
    fn verify(
        &self,
        msg: &[u8],
        signature: &KeyEvolvingSignature<'a, CompactSignature<CompactSignature<S, KP>, KP>>,
    ) -> Result<(), signature::Error> {
        let vkey = signature.to_verifying_key();
        if &vkey != self {
            return Err(signature::Error::new());
        }

        signature.verify_helper(msg)
    }
}

impl<'a, S, KP, H>
    Verifier<
        KeyEvolvingSignature<'a, CompactSignature<CompactSignature<S, KP>, sum::Double<KP, H>>>,
    > for sum::VerifyingKey<H>
where
    KP: KeypairRef<VerifyingKey: AsRef<[u8]>> + KeyInit + Evolve,
    H: Digest,
    KeyEvolvingSignature<'a, CompactSignature<S, KP>>: ToVerifyingKey<H> + VerifierHelper,
{
    fn verify(
        &self,
        msg: &[u8],
        signature: &KeyEvolvingSignature<
            'a,
            CompactSignature<CompactSignature<S, KP>, sum::Double<KP, H>>,
        >,
    ) -> Result<(), signature::Error> {
        let vkey = signature.to_verifying_key();
        if &vkey != self {
            return Err(signature::Error::new());
        }

        signature.verify_helper(msg)
    }
}

impl<'a, S, KP, H> Verifier<KeyEvolvingSignature<'a, CompactSignature<CompactSignature<S, KP>, KP>>>
    for sum::Double<KP, H>
where
    KP: KeypairRef + KeySizeUser + Evolve,
    KP::VerifyingKey: AsRef<[u8]> + Verifier<KeyEvolvingSignature<'a, S>>,
    H: Digest,
{
    fn verify(
        &self,
        msg: &[u8],
        signature: &KeyEvolvingSignature<'a, CompactSignature<CompactSignature<S, KP>, KP>>,
    ) -> Result<(), signature::Error> {
        self.verifying_key().verify(msg, signature)
    }
}

impl<'a, S, KP, H>
    Verifier<
        KeyEvolvingSignature<'a, CompactSignature<CompactSignature<S, KP>, sum::Double<KP, H>>>,
    > for sum::Pow2<KP, H>
where
    KP: KeypairRef<VerifyingKey: AsRef<[u8]>> + KeyInit + Evolve,
    H: Digest,
    KeyEvolvingSignature<'a, CompactSignature<S, KP>>: ToVerifyingKey<H> + VerifierHelper,
{
    fn verify(
        &self,
        msg: &[u8],
        signature: &KeyEvolvingSignature<
            'a,
            CompactSignature<CompactSignature<S, KP>, sum::Double<KP, H>>,
        >,
    ) -> Result<(), signature::Error> {
        self.verifying_key().verify(msg, signature)
    }
}

pub type DoubleCompactSignature<S, KP> = CompactSignature<CompactSignature<S, KP>, KP>;
pub type Pow2CompactSignature<S, KP, H> =
    CompactSignature<DoubleCompactSignature<S, KP>, sum::Double<KP, H>>;
pub type Pow3CompactSignature<S, KP, H> =
    CompactSignature<Pow2CompactSignature<S, KP, H>, sum::Pow2<KP, H>>;
pub type Pow4CompactSignature<S, KP, H> =
    CompactSignature<Pow3CompactSignature<S, KP, H>, sum::Pow3<KP, H>>;
pub type Pow5CompactSignature<S, KP, H> =
    CompactSignature<Pow4CompactSignature<S, KP, H>, sum::Pow4<KP, H>>;
pub type Pow6CompactSignature<S, KP, H> =
    CompactSignature<Pow5CompactSignature<S, KP, H>, sum::Pow5<KP, H>>;

/// Needed to verify only the leaf signature against its verifying key.
trait VerifierHelper {
    fn verify_helper(&self, msg: &[u8]) -> signature::Result<()>;
}

impl<'a, S, KP>
    VerifierHelper
    for KeyEvolvingSignature<'a, CompactSignature<CompactSignature<S, KP>, KP>>
where
    KP: KeypairRef + Evolve,
    KP::VerifyingKey: Verifier<KeyEvolvingSignature<'a, S>>,
{
    fn verify_helper(
        &self,
        msg: &[u8],
    ) -> signature::Result<()> {
        let kes = KeyEvolvingSignature {
            signature: &self.signature.signature.signature,
            period: self.period % KP::PERIOD_COUNT,
        };

        self.signature.signature.vkey.verify(msg, &kes)
    }
}

impl<'a, S, KP, H>
    VerifierHelper for KeyEvolvingSignature<'a, CompactSignature<CompactSignature<S, KP>, sum::Double<KP, H>>>
where
    KP: KeypairRef + KeyInit + Evolve,
    H: OutputSizeUser,
    KeyEvolvingSignature<'a, CompactSignature<S, KP>>: VerifierHelper,
{
    fn verify_helper(
        &self,
        msg: &[u8],
    ) -> signature::Result<()> {
        let kes = KeyEvolvingSignature {
            signature: &self.signature.signature,
            period: self.period % sum::Double::<KP, H>::PERIOD_COUNT,
        };

        kes.verify_helper(msg)
    }
}

#[cfg(test)]
mod tests {
    use digest::KeyInit;
    use rand::random;
    use signature::Signer;

    use super::*;
    use crate::{
        sum, tests::{KeyBase, THash}, Evolve
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

        let key: sum::Double<KeyBase, THash> = sum::Double::new(&seed.into());
        let signature: DoubleCompactSignature<_, _> = key.sign(msg);
        let kes = KeyEvolvingSignature {
            signature: &signature,
            period: key.period(),
        };
        let vkey = kes.to_verifying_key();
        assert_eq!(vkey, key.verifying_key());

        let key: sum::Pow6<KeyBase, THash> = sum::Pow6::new(&seed.into());
        let signature: Pow6CompactSignature<_, _, _> = key.sign(msg);
        let kes = KeyEvolvingSignature {
            signature: &signature,
            period: key.period(),
        };
        let vkey = kes.to_verifying_key();
        assert_eq!(vkey, key.verifying_key());
    }

    #[test]
    fn verify() {
        let seed: [u8; 32] = random();
        let msg = b"Hello, world!";

        let key: sum::Pow2<KeyBase, THash> = sum::Double::new(&seed.into());
        let signature: Pow2CompactSignature<_, _, _> = key.sign(msg);
        let kes = KeyEvolvingSignature {
            signature: &signature,
            period: key.period(),
        };
        assert!(key.verifying_key().verify(msg, &kes).is_ok());
    }

    
    #[test]
    fn can_verify_from_all_evolutions() {
        let key: [u8; 32] = random();
        let mut pow6: sum::Pow6<KeyBase, THash> = sum::Pow6::new(&key.into());

        let vkey = pow6.verifying_key();
        let raw_signature: Pow6CompactSignature<_, _, _> =
            pow6.try_sign(MESSAGES[0]).unwrap();
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
        let mut kes1: sum::Pow6<KeyBase, THash> = sum::Pow6::new(&key.into());
        let other_vkey =
            sum::Pow6::<KeyBase, THash>::new(&random::<[u8; 32]>().into()).verifying_key();

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
        let mut skey: sum::Pow6<KeyBase, THash> = sum::Pow6::new(&key.into());

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
