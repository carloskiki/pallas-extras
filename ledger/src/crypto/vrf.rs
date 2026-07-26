use curve25519_dalek::{
    edwards::{CompressedEdwardsY, EdwardsPoint},
    scalar::Scalar,
};
use ed25519_dalek::{SecretKey, hazmat::ExpandedSecretKey, pkcs8::PublicKeyBytes};
use sha2::{Digest, Sha512};
use signature::{Signer, Verifier};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

const CHALLENGE_LENGTH: usize = 16;
const POINT_LENGTH: usize = 32;
const SCALAR_LENGTH: usize = 32;
pub const PROOF_LENGTH: usize = POINT_LENGTH + CHALLENGE_LENGTH + SCALAR_LENGTH;
const HASH_TO_CURVE_SUITE: &[u8] = b"edwards25519_XMD:SHA-512_ELL2_NU_";
const SUITE: u8 = 0x04;
const DST: &[&[u8]] = &[b"ECVRF_", HASH_TO_CURVE_SUITE, &[SUITE]];

pub type Hash = [u8; 64];

/// The proving key.
///
/// Implements [`Signer`] to produce a [`Proof`].
#[derive(Debug)]
pub struct ProvingKey {
    secret: ExpandedSecretKey,
    verifying_key: VerifyingKey,
}

impl From<&SecretKey> for ProvingKey {
    fn from(secret: &SecretKey) -> Self {
        let secret = ExpandedSecretKey::from(secret);
        let verifying_key = VerifyingKey((&secret).into());
        ProvingKey {
            secret,
            verifying_key,
        }
    }
}

/// The verifying key.
///
/// Implements [`Verifier`] to verify a [`Proof`].
#[derive(Debug, Clone)]
pub struct VerifyingKey(
    /// INVARIANT: the verifying key must not be on a small order subgroup.
    ed25519_dalek::VerifyingKey,
);

impl TryFrom<&PublicKeyBytes> for VerifyingKey {
    type Error = ed25519_dalek::pkcs8::spki::Error;

    fn try_from(value: &PublicKeyBytes) -> Result<Self, Self::Error> {
        // FIXME: Same as for gamma decode, need RFC 8032's "decode".
        let verifying_key = ed25519_dalek::VerifyingKey::try_from(value)?;
        if verifying_key.is_weak() {
            return Err(ed25519_dalek::pkcs8::spki::Error::KeyMalformed);
        }
        Ok(Self(verifying_key))
    }
}

/// A VRF proof.
///
/// Can be turned into a hash output using [`Proof::to_hash`].
#[repr(C)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    FromBytes,
    IntoBytes,
    Unaligned,
    KnownLayout,
    Immutable,
)]
pub struct Proof {
    gamma: [u8; POINT_LENGTH],
    challenge: [u8; CHALLENGE_LENGTH],
    scalar: [u8; SCALAR_LENGTH],
}

impl From<[u8; PROOF_LENGTH]> for Proof {
    fn from(bytes: [u8; PROOF_LENGTH]) -> Self {
        let (gamma, challenge, scalar) = zerocopy::transmute!(bytes);
        Proof {
            gamma,
            challenge,
            scalar,
        }
    }
}

impl Proof {
    fn parts(&self) -> Option<(EdwardsPoint, Scalar, Scalar)> {
        // FIXME: The RFC states that we need RFC 8032's "decode", not ZIP-215:
        // https://github.com/dalek-cryptography/curve25519-dalek/pull/833.
        let gamma = CompressedEdwardsY::from_slice(&self.gamma)
            .expect("slice length is correct")
            .decompress()?;
        let mut c_scalar_string = [0; SCALAR_LENGTH];
        c_scalar_string[..CHALLENGE_LENGTH].copy_from_slice(&self.challenge);
        let c = Scalar::from_canonical_bytes(c_scalar_string).expect("16 bytes scalar is reduced");
        let s = Scalar::from_canonical_bytes(self.scalar).into_option()?;
        Some((gamma, c, s))
    }

    /// Get the hash associated with this proof.
    pub fn to_hash(&self) -> Option<Hash> {
        const DOMAIN_SEPARATOR_FRONT: u8 = 0x03;
        const DOMAIN_SEPARATOR_BACK: u8 = 0x00;

        let (gamma, _, _) = self.parts()?;
        let mut hasher = Sha512::new();
        hasher.update([SUITE]);
        hasher.update([DOMAIN_SEPARATOR_FRONT]);
        hasher.update(gamma.mul_by_cofactor().compress().as_bytes());
        hasher.update([DOMAIN_SEPARATOR_BACK]);
        Some(hasher.finalize().into())
    }
}

impl Signer<Proof> for ProvingKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Proof, signature::Error> {
        let h =
            EdwardsPoint::encode_to_curve::<Sha512>(&[self.verifying_key.0.as_bytes(), msg], DST);
        let h_string = h.compress().0;
        let gamma = self.secret.scalar * h;
        let gamma_string = gamma.compress().0;
        let k_string: [u8; _] = Sha512::new()
            .chain_update(self.secret.hash_prefix)
            .chain_update(h_string)
            .finalize()
            .into();
        let k = Scalar::from_bytes_mod_order_wide(&k_string);
        let (c, c_string) = challenge([
            self.verifying_key.0.as_bytes(),
            &h_string,
            &gamma_string,
            EdwardsPoint::mul_base(&k).compress().as_bytes(),
            (k * h).compress().as_bytes(),
        ]);
        let s = k + c * self.secret.scalar;
        Ok(Proof {
            gamma: gamma_string,
            challenge: c_string,
            scalar: s.to_bytes(),
        })
    }
}

impl Verifier<Proof> for VerifyingKey {
    fn verify(&self, msg: &[u8], proof: &Proof) -> Result<(), signature::Error> {
        let (gamma, c, s) = proof.parts().ok_or(signature::Error::new())?;
        let h = EdwardsPoint::encode_to_curve::<Sha512>(&[self.0.as_bytes(), msg], DST);
        let u = EdwardsPoint::mul_base(&s) + (-c * self.0.to_edwards());
        let v = (s * h) + (-c * gamma);
        let (c_prime, _) = challenge([
            self.0.as_bytes(),
            h.compress().as_bytes(),
            &proof.gamma,
            u.compress().as_bytes(),
            v.compress().as_bytes(),
        ]);

        if c == c_prime {
            Ok(())
        } else {
            Err(signature::Error::new())
        }
    }
}

fn challenge(points: [&[u8; 32]; 5]) -> (Scalar, [u8; CHALLENGE_LENGTH]) {
    const DOMAIN_SEPARATOR_FRONT: u8 = 0x02;
    const DOMAIN_SEPARATOR_BACK: u8 = 0x00;

    let mut hasher = Sha512::new();
    hasher.update([SUITE]);
    hasher.update([DOMAIN_SEPARATOR_FRONT]);
    for point in points {
        hasher.update(point);
    }
    hasher.update([DOMAIN_SEPARATOR_BACK]);
    let mut hash: [u8; _] = hasher.finalize().into();
    hash[CHALLENGE_LENGTH..].fill(0);
    (
        Scalar::from_canonical_bytes(std::array::from_fn(|i| hash[i]))
            .expect("16 bytes scalar is reduced"),
        std::array::from_fn(|i| hash[i]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use const_hex::const_decode_to_array;

    struct Vector {
        secret_key: [u8; 32],
        verifying_key: [u8; 32],
        alpha: &'static [u8],
        proof: [u8; PROOF_LENGTH],
        hash: Hash,
    }

    const fn unwrap<T: Copy>(r: Result<T, const_hex::FromHexError>) -> T {
        match r {
            Ok(value) => value,
            Err(_) => panic!("hex decoding error"),
        }
    }

    macro_rules! vector {
        (
        secret_key: $secret_key:literal,
        verifying_key: $verifying_key:literal,
        alpha($len:literal): $alpha:literal,
        proof: $proof:literal,
        hash: $hash:literal $(,)?
        ) => {
            Vector {
                secret_key: unwrap(const_decode_to_array($secret_key)),
                verifying_key: unwrap(const_decode_to_array($verifying_key)),
                alpha: &unwrap(const_decode_to_array::<$len>($alpha)),
                proof: unwrap(const_decode_to_array($proof)),
                hash: unwrap(const_decode_to_array($hash)),
            }
        };
    }
    const VECTORS: &[Vector] = &[
        vector!(
            secret_key: b"9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60",
            verifying_key: b"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
            alpha(0): b"",
            proof: b"7d9c633ffeee27349264cf5c667579fc583b4bda63ab71d001f89c10003ab46f14adf9a3cd8b8412d9038531e865c341cafa73589b023d14311c331a9ad15ff2fb37831e00f0acaa6d73bc9997b06501",
            hash: b"9d574bf9b8302ec0fc1e21c3ec5368269527b87b462ce36dab2d14ccf80c53cccf6758f058c5b1c856b116388152bbe509ee3b9ecfe63d93c3b4346c1fbc6c54"
        ),
        vector!(
            secret_key: b"4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb",
            verifying_key: b"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
            alpha(1): b"72",
            proof: b"47b327393ff2dd81336f8a2ef10339112401253b3c714eeda879f12c509072ef055b48372bb82efbdce8e10c8cb9a2f9d60e93908f93df1623ad78a86a028d6bc064dbfc75a6a57379ef855dc6733801",
            hash: b"38561d6b77b71d30eb97a062168ae12b667ce5c28caccdf76bc88e093e4635987cd96814ce55b4689b3dd2947f80e59aac7b7675f8083865b46c89b2ce9cc735",
        ),
        vector!(
            secret_key: b"c5aa8df43f9f837bedb7442f31dcb7b166d38535076f094b85ce3a2e0b4458f7",
            verifying_key: b"fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025",
            alpha(2): b"af82",
            proof: b"926e895d308f5e328e7aa159c06eddbe56d06846abf5d98c2512235eaa57fdce35b46edfc655bc828d44ad09d1150f31374e7ef73027e14760d42e77341fe05467bb286cc2c9d7fde29120a0b2320d04",
            hash: b"121b7f9b9aaaa29099fc04a94ba52784d44eac976dd1a3cca458733be5cd090a7b5fbd148444f17f8daf1fb55cb04b1ae85a626e30a54b4b0f8abf4a43314a58",
        ),
    ];

    #[test]
    fn rfc() {
        for vector in VECTORS {
            let proving_key = ProvingKey::from(&vector.secret_key);
            let verifying_key =
                VerifyingKey::try_from(&PublicKeyBytes(vector.verifying_key)).unwrap();
            let proof = proving_key.try_sign(vector.alpha).unwrap();
            assert_eq!(proof, Proof::from(vector.proof));
            assert_eq!(proof.to_hash().unwrap(), vector.hash);
            verifying_key.verify(vector.alpha, &proof).unwrap();
        }
    }
}
