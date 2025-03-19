//! The sum construction from the MMM paper, with convinient type aliases.
//!
//! Items in this module should not be directly imported as they have very generic names such as
//! [`Signature`] or [`VerifyingKey`]. Instead, import the module and use is as a namespace (e.g.,
//! `sum::Signature`).

mod compact;

use blake2::Blake2b;
use digest::{
    Digest, KeyInit, OutputSizeUser,
    consts::U64,
    crypto_common::KeySizeUser,
    generic_array::{ArrayLength, GenericArray},
    typenum::{IsLessOrEqual, LeEq, NonZero},
};
use ref_cast::RefCast;
use either::Either::{self, Left, Right};
use signature::{Keypair, KeypairRef, SignatureEncoding, Signer, Verifier};
use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::{Evolve, KeyEvolvingSignature};

pub use compact::*;

/// Sum construction from the MMM paper.
///
/// Given two evolving keys `L` and `R`, and an hash function `H`, we construct a new evolving key
/// that has `L::PERIOD_COUNT + R::PERIOD_COUNT` periods. The verifying key is the hash of the
/// concatenation of the verifying keys of `L` and `R`, using `H`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sum<L, R, H>
where
    L: KeypairRef,
    R: KeySizeUser + KeypairRef,
    H: OutputSizeUser,
{
    inner: Either<(L, R::VerifyingKey), (R, L::VerifyingKey)>,
    seed: GenericArray<u8, R::KeySize>,
    vkey: GenericArray<u8, H::OutputSize>,
}

impl<L, R, H> AsRef<VerifyingKey<L, R, H>> for Sum<L, R, H>
where
    L: KeypairRef,
    R: KeySizeUser + KeypairRef,
    H: OutputSizeUser,
{
    fn as_ref(&self) -> &VerifyingKey<L, R, H> {
        VerifyingKey::ref_cast(&self.vkey)
    }
}

impl<L, R, H> KeypairRef for Sum<L, R, H>
where
    L: KeypairRef,
    R: KeySizeUser + KeypairRef,
    H: OutputSizeUser,
{
    type VerifyingKey = VerifyingKey<L, R, H>;
}

impl<L, R, H> KeySizeUser for Sum<L, R, H>
where
    L: KeySizeUser + KeypairRef,
    R: KeySizeUser<KeySize = L::KeySize> + KeypairRef,
    H: OutputSizeUser,
{
    type KeySize = L::KeySize;
}

impl<L, R, H> KeyInit for Sum<L, R, H>
where
    L: KeypairRef<VerifyingKey: AsRef<[u8]>> + KeyInit + KeySizeUser<KeySize = R::KeySize>,
    R: KeyInit + KeypairRef<VerifyingKey: AsRef<[u8]>>,
    R::KeySize: IsLessOrEqual<Blake2bMaxSize>,
    LeEq<R::KeySize, Blake2bMaxSize>: NonZero,
    H: Digest,
{
    fn new(key: &digest::Key<Self>) -> Self {
        let (left, right) = double_length(key);
        let left_key = L::new(&left);
        let right_key = R::new(&right);
        let mut vkey_hasher = H::new();
        vkey_hasher.update(left_key.verifying_key());
        vkey_hasher.update(right_key.verifying_key());
        let vkey_key = vkey_hasher.finalize();
        Sum {
            inner: Left((left_key, right_key.verifying_key())),
            seed: right,
            vkey: vkey_key,
        }
    }
}

impl<S, L, R, H> Signer<Signature<S, L, R>> for Sum<L, R, H>
where
    L: Signer<S> + KeypairRef,
    R: Signer<S> + KeySizeUser + KeypairRef,
    H: OutputSizeUser,
{
    fn try_sign(&self, msg: &[u8]) -> Result<Signature<S, L, R>, signature::Error> {
        match &self.inner {
            Left((left, right_vkey)) => Ok(Signature {
                signature: left.try_sign(msg)?,
                left_vkey: left.verifying_key(),
                right_vkey: right_vkey.clone(),
            }),
            Right((right, left_vkey)) => Ok(Signature {
                signature: right.try_sign(msg)?,
                left_vkey: left_vkey.clone(),
                right_vkey: right.verifying_key(),
            }),
        }
    }
}

impl<'a, S, L, R, H> Verifier<KeyEvolvingSignature<'a, Signature<S, L, R>>> for Sum<L, R, H>
where
    L: KeypairRef + Evolve,
    L::VerifyingKey: Verifier<KeyEvolvingSignature<'a, S>> + AsRef<[u8]>,
    R: KeySizeUser + KeypairRef,
    R::VerifyingKey: Verifier<KeyEvolvingSignature<'a, S>> + AsRef<[u8]>,
    H: Digest,
{
    fn verify(
        &self,
        msg: &[u8],
        signature: &KeyEvolvingSignature<'a, Signature<S, L, R>>,
    ) -> Result<(), signature::Error> {
        self.verifying_key().verify(msg, signature)
    }
}

impl<L, R, H> Evolve for Sum<L, R, H>
where
    L: KeypairRef + Evolve,
    R: KeyInit + KeypairRef + Evolve,
    H: OutputSizeUser,
{
    const PERIOD_COUNT: u32 = L::PERIOD_COUNT + R::PERIOD_COUNT;

    fn evolve(self) -> Option<Self> {
        match self.inner {
            Left((left, right_vkey)) => {
                let left_vkey = left.verifying_key();
                Some(if let Some(left) = left.evolve() {
                    Sum {
                        inner: Left((left, right_vkey)),
                        seed: self.seed,
                        vkey: self.vkey,
                    }
                } else {
                    let right = R::new(&self.seed);
                    Sum {
                        inner: Right((right, left_vkey)),
                        seed: Default::default(),
                        vkey: self.vkey,
                    }
                })
            }
            Right((right, left_vkey)) => right.evolve().map(|right| Sum {
                inner: Right((right, left_vkey)),
                seed: self.seed,
                vkey: self.vkey,
            }),
        }
    }

    fn period(&self) -> u32 {
        match &self.inner {
            Left((left, _)) => left.period(),
            Right((right, _)) => L::PERIOD_COUNT + right.period(),
        }
    }
}

/// Signature for the sum construction.
///
/// When both the left and right parts of the sum are the same type (`Sum<T, T, H>`), one can use
/// the [`CompactSignature`] type instead to have a more compact representation.
///
/// This can be chosen as the output of [`Sum::sign`] by setting the `Signer<S>` type parameter.
/// To [`Verifier::verify`] this signature, it must first be assembled into a
/// [`KeyEvolvingSignature`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Signature<S, L, R>
where
    L: KeypairRef,
    R: KeypairRef,
{
    pub signature: S,
    pub left_vkey: L::VerifyingKey,
    pub right_vkey: R::VerifyingKey,
}

impl<'a, S, L, R> TryFrom<&'a [u8]> for Signature<S, L, R>
where
    S: TryFrom<&'a [u8]>,
    L: KeypairRef<VerifyingKey: TryFrom<&'a [u8]> + KeySizeUser>,
    R: KeypairRef<VerifyingKey: TryFrom<&'a [u8]> + KeySizeUser>,
{
    type Error = SignatureFromBytesError<
        <S as TryFrom<&'a [u8]>>::Error,
        <L::VerifyingKey as TryFrom<&'a [u8]>>::Error,
        <R::VerifyingKey as TryFrom<&'a [u8]>>::Error,
    >;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let left_size = L::VerifyingKey::key_size();
        let right_size = R::VerifyingKey::key_size();

        let signature_end = value.len().saturating_sub(left_size + right_size);
        let left_vkey_end = value.len().saturating_sub(right_size);

        let signature = S::try_from(value).map_err(SignatureFromBytesError::Signature)?;
        let left_vkey = L::VerifyingKey::try_from(&value[signature_end..left_vkey_end])
            .map_err(SignatureFromBytesError::Left)?;
        let right_vkey = R::VerifyingKey::try_from(&value[left_vkey_end..])
            .map_err(SignatureFromBytesError::Right)?;
        Ok(Signature {
            signature,
            left_vkey,
            right_vkey,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SignatureFromBytesError<SE, LKE, RKE> {
    Signature(SE),
    Left(LKE),
    Right(RKE),
}

impl<S, L, R> Display for SignatureFromBytesError<S, L, R>
where
    S: Display,
    L: Display,
    R: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureFromBytesError::Signature(e) => write!(f, "Signature error: {}", e),
            SignatureFromBytesError::Left(e) => write!(f, "Left verifying key error: {}", e),
            SignatureFromBytesError::Right(e) => write!(f, "Right verifying key error: {}", e),
        }
    }
}

impl<S, L, R> Error for SignatureFromBytesError<S, L, R> where
    SignatureFromBytesError<S, L, R>: Debug + Display
{
}

impl<S, L, R> From<Signature<S, L, R>> for Vec<u8>
where
    L: KeypairRef<VerifyingKey: AsRef<[u8]>>,
    R: KeypairRef<VerifyingKey: AsRef<[u8]>>,
    S: SignatureEncoding,
{
    fn from(
        Signature {
            signature,
            left_vkey,
            right_vkey,
        }: Signature<S, L, R>,
    ) -> Self {
        let mut storage = signature.to_vec();
        storage.extend_from_slice(left_vkey.as_ref());
        storage.extend_from_slice(right_vkey.as_ref());
        storage
    }
}

impl<S, L, R> SignatureEncoding for Signature<S, L, R>
where
    S: SignatureEncoding,
    L: KeypairRef<VerifyingKey: AsRef<[u8]> + for<'b> TryFrom<&'b [u8]> + KeySizeUser> + Clone,
    R: KeypairRef<VerifyingKey: AsRef<[u8]> + for<'b> TryFrom<&'b [u8]> + KeySizeUser> + Clone,
{
    // We really need generic_const_expr here to have something stored on the stack.
    type Repr = Vec<u8>;
}

/// Verifying key for the sum construction.
///
/// Internally this is simply the hash of the concatenation of the verifying keys of the left and
/// right parts of the sum.
#[derive(RefCast)]
#[repr(transparent)] 
pub struct VerifyingKey<L, R, H>(
    GenericArray<u8, H::OutputSize>,
    // We need this to bound recursion in the type checker. Otherwise, typecheck goes into infinite
    // recursion, and gives nasty error messages. Sad reality.
    std::marker::PhantomData<(L, R)>,
)
where
    H: OutputSizeUser;

impl<L, R, H: OutputSizeUser> Clone for VerifyingKey<L, R, H> {
    fn clone(&self) -> Self {
        VerifyingKey(self.0.clone(), std::marker::PhantomData)
    }
}

impl<L, R, H: OutputSizeUser> AsRef<[u8]> for VerifyingKey<L, R, H> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<L, R, H: OutputSizeUser> PartialEq for VerifyingKey<L, R, H> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<L, R, H: OutputSizeUser> Eq for VerifyingKey<L, R, H> {}

impl<L, R, H: OutputSizeUser> PartialOrd for VerifyingKey<L, R, H> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl<L, R, H: OutputSizeUser> Ord for VerifyingKey<L, R, H> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<L, R, H: OutputSizeUser> Debug for VerifyingKey<L, R, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SumVerifyingKey").field(&self.0).finish()
    }
}

impl<L, R, H: OutputSizeUser> KeySizeUser for VerifyingKey<L, R, H> {
    type KeySize = H::OutputSize;
}

impl<L, R, H: OutputSizeUser> From<digest::Key<Self>> for VerifyingKey<L, R, H>
where
    L: KeypairRef,
    R: KeySizeUser + KeypairRef,
    H: OutputSizeUser,
{
    fn from(key: digest::Key<Self>) -> Self {
        VerifyingKey(key, std::marker::PhantomData)
    }
}

impl<'a, S, L, R, H> Verifier<KeyEvolvingSignature<'a, Signature<S, L, R>>>
    for VerifyingKey<L, R, H>
where
    L: KeypairRef + Evolve,
    L::VerifyingKey: Verifier<KeyEvolvingSignature<'a, S>> + AsRef<[u8]>,
    R: KeySizeUser + KeypairRef,
    R::VerifyingKey: Verifier<KeyEvolvingSignature<'a, S>> + AsRef<[u8]>,
    H: Digest,
{
    fn verify(
        &self,
        msg: &[u8],
        KeyEvolvingSignature {
            signature:
                Signature {
                    signature,
                    left_vkey,
                    right_vkey,
                },
            period,
        }: &KeyEvolvingSignature<'a, Signature<S, L, R>>,
    ) -> Result<(), signature::Error> {
        let mut inner_signature = KeyEvolvingSignature {
            signature,
            period: *period,
        };
        if H::new()
            .chain_update(left_vkey)
            .chain_update(right_vkey)
            .finalize()
            == self.0
        {
            if *period < L::PERIOD_COUNT {
                left_vkey.verify(msg, &inner_signature)
            } else {
                inner_signature.period -= L::PERIOD_COUNT;
                right_vkey.verify(msg, &inner_signature)
            }
        } else {
            Err(signature::Error::new())
        }
    }
}

// This does not work for some reason: `<Blake2bVarCore as OutputSizeUser>::OutputSize`
// But I am certain that they are the same.
type Blake2bMaxSize = U64;

/// Function used by input-output-hk's MMM implementation.
fn double_length<U>(key: &GenericArray<u8, U>) -> (GenericArray<u8, U>, GenericArray<u8, U>)
where
    LeEq<U, Blake2bMaxSize>: NonZero,
    U: ArrayLength<u8> + IsLessOrEqual<Blake2bMaxSize>,
{
    let mut hasher: Blake2b<U> = <Blake2b<U> as Digest>::new();
    hasher.update([1]);
    hasher.update(key);
    let left = hasher.finalize_reset();
    hasher.update([2]);
    hasher.update(key);
    let right = hasher.finalize();

    (left, right)
}

/// Summation of the same type.
pub type Double<T, H> = Sum<T, T, H>;
/// Signature of the summation of the same type.
pub type DoubleSignature<S, T> = Signature<S, T, T>;
pub type DoubleVerifyingKey<T, H> = VerifyingKey<T, T, H>;

/// Repeated sum of the same type with `2^2` periods.
pub type Pow2<T, H> = Double<Double<T, H>, H>;
/// Signature of the repeated sum of the same type with `2^2` periods.
pub type Pow2Signature<S, T, H> = DoubleSignature<DoubleSignature<S, T>, Double<T, H>>;
pub type Pow2VerifyingKey<T, H> = DoubleVerifyingKey<Double<T, H>, H>;

/// Repeated sum of the same type with `2^3` periods.
pub type Pow3<T, H> = Double<Pow2<T, H>, H>;
/// Signature of the repeated sum of the same type with `2^3` periods.
pub type Pow3Signature<S, T, H> = DoubleSignature<Pow2Signature<S, T, H>, Pow2<T, H>>;
pub type Pow3VerifyingKey<T, H> = DoubleVerifyingKey<Pow2<T, H>, H>;

/// Repeated sum of the same type with `2^4` periods.
pub type Pow4<T, H> = Double<Pow3<T, H>, H>;
/// Signature of the repeated sum of the same type with `2^4` periods.
pub type Pow4Signature<S, T, H> = DoubleSignature<Pow3Signature<S, T, H>, Pow3<T, H>>;
pub type Pow4VerifyingKey<T, H> = DoubleVerifyingKey<Pow3<T, H>, H>;

/// Repeated sum of the same type with `2^5` periods.
pub type Pow5<T, H> = Double<Pow4<T, H>, H>;
/// Signature of the repeated sum of the same type with `2^5` periods.
pub type Pow5Signature<S, T, H> = DoubleSignature<Pow4Signature<S, T, H>, Pow4<T, H>>;
pub type Pow5VerifyingKey<T, H> = DoubleVerifyingKey<Pow4<T, H>, H>;

/// Repeated sum of the same type with `2^6` periods.
pub type Pow6<T, H> = Double<Pow5<T, H>, H>;
/// Signature of the repeated sum of the same type with `2^6` periods.
pub type Pow6Signature<S, T, H> = DoubleSignature<Pow5Signature<S, T, H>, Pow5<T, H>>;
pub type Pow6VerifyingKey<T, H> = DoubleVerifyingKey<Pow5<T, H>, H>;

#[cfg(test)]
mod tests {
    use blake2::Blake2b;
    use digest::{KeyInit, consts::U32};
    use ed25519_dalek::Signature;
    use rand::random;
    use signature::{Keypair, Signer, Verifier};

    use crate::{
        Evolve, KeyEvolvingSignature,
        sum::{Pow6, Pow6Signature},
        tests::KeyBase,
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
    fn update_count() {
        let key: [u8; 32] = random();
        let mut pow6: Pow6<KeyBase, Blake2b<U32>> = Pow6::new(&key.into());

        assert_eq!(pow6.period(), 0);
        for period in 1..Pow6::<KeyBase, Blake2b<U32>>::PERIOD_COUNT {
            pow6 = pow6.evolve().unwrap();
            assert_eq!(pow6.period(), period);
        }
        assert!(pow6.evolve().is_none());
    }

    #[test]
    fn always_same_vkey() {
        let key: [u8; 32] = random();
        let mut pow6: Pow6<KeyBase, Blake2b<U32>> = Pow6::new(&key.into());

        let mut vkey = pow6.verifying_key();
        for _ in 1..Pow6::<KeyBase, Blake2b<U32>>::PERIOD_COUNT {
            pow6 = pow6.evolve().unwrap();
            assert_eq!(vkey, pow6.verifying_key());
            vkey = pow6.verifying_key();
        }
    }

    #[test]
    fn can_verify_from_all_evolutions() {
        let key: [u8; 32] = random();
        let mut pow6: Pow6<KeyBase, Blake2b<U32>> = Pow6::new(&key.into());

        let vkey = pow6.verifying_key();
        let raw_signature: Pow6Signature<Signature, KeyBase, Blake2b<U32>> =
            pow6.try_sign(MESSAGES[0]).unwrap();
        let mut signature = KeyEvolvingSignature {
            signature: &raw_signature,
            period: pow6.period(),
        };
        assert!(vkey.verify(MESSAGES[0], &signature).is_ok());

        for i in 1..Pow6::<KeyBase, Blake2b<U32>>::PERIOD_COUNT {
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
        let mut kes1: Pow6<KeyBase, Blake2b<U32>> = Pow6::new(&key.into());
        let other_vkey =
            Pow6::<KeyBase, Blake2b<U32>>::new(&random::<[u8; 32]>().into()).verifying_key();

        for msg in MESSAGES {
            let raw_signature: Pow6Signature<_, _, _> = kes1.try_sign(msg).unwrap();
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
        let mut skey: Pow6<KeyBase, Blake2b<U32>> = Pow6::new(&key.into());

        for msg in MESSAGES {
            let signature: Pow6Signature<_, _, _> = skey.try_sign(msg).unwrap();
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

    #[test]
    #[should_panic]
    fn signature_encoding_roundtrip() {
        todo!(
            "we need our patches to ed25519-dalek to make this work - \
            alternatively we can fork it since it is only a dev-dependency"
        );
    }
}
