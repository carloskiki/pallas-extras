use bip39::entropy::Entropy;
use curve25519_dalek::{edwards::EdwardsPoint, Scalar, edwards::CompressedEdwardsY, scalar::clamp_integer};
use digest::consts::{U28, U32};
use hmac::Hmac;
use minicbor::{
    Decode, Decoder, Encode, Encoder, decode,
    encode::{self, Write},
};
use pbkdf2::pbkdf2_hmac;
use sha2::{Digest, Sha512};
use zerocopy::transmute;

pub mod byron;
pub mod cbor;
pub mod network;
pub mod shelley;

pub type Blake2b224 = blake2::Blake2b<U28>;
type Blake2b224Digest = [u8; 28];

pub type Blake2b256 = blake2::Blake2b<U32>;
type Blake2b256Digest = [u8; 32];

pub type VerifyingKey = CompressedEdwardsY;

pub struct HardIndex(u32);

impl HardIndex {
    /// Generates a hardened index from the given `u32`. The MSB is forced to 1.
    pub fn new(index: u32) -> Self {
        Self(index | (1 << 31))
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

pub struct SoftIndex(u32);

impl SoftIndex {
    /// Generates a soft index from the given `u32`. The MSB is forced to 0.
    pub fn new(index: u32) -> Self {
        Self(index & !(1 << 31))
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

pub struct ExtendedSecretKey {
    // Invariant: Must be a valid scalar (clamped).
    pub key: [u8; 32],
    pub hash_prefix: [u8; 32],
    pub chain_code: [u8; 32],
}

impl ExtendedSecretKey {
    /// Generate an [`ExtendedSecretKey`] from [`Entropy`] and a password.
    ///
    /// This uses the Icarus method to generate the key, which is the method recommended by
    /// [CIP3](https://github.com/cardano-foundation/CIPs/tree/master/CIP-0003).
    pub fn from_entropy_icarus(entropy: Entropy, password: &[u8]) -> Self {
        let mut output = [0; 96];
        const ITERATION_COUNT: u32 = 4096;
        pbkdf2_hmac::<Sha512>(password, entropy.as_ref(), ITERATION_COUNT, &mut output);
        let [mut key, hash_prefix, chain_code]: [[u8; 32]; 3] = transmute!(output);
        key[31] &= 0b1101_1111;
        let key = clamp_integer(key);
        Self {
            key,
            hash_prefix,
            chain_code,
        }
    }

    /// Generate an [`ExtendedSecretKey`] from a 32 bytes secret and chain code.
    ///
    /// This follows the method defined in [BIP32-Ed25519 Hierarchical Deterministic Keys over a
    /// Non-linear Keyspace](https://input-output-hk.github.io/adrestia/static/Ed25519_BIP.pdf)
    pub fn from_nonextended(master: [u8; 32], chain_code: [u8; 32]) -> Self {
        let full_key: [u8; 64] = Sha512::digest(master).into();
        let [mut key, hash_prefix]: [[u8; 32]; 2] = transmute!(full_key);
        key[31] &= 0b1101_1111;
        let key = clamp_integer(key);
        Self {
            key,
            hash_prefix,
            chain_code,
        }
    }

    pub fn derive_child(&self, index: HardIndex) -> Self {
        use digest::{FixedOutput, KeyInit, Update};
        let mut key_hmac: Hmac<Sha512> = hmac::Hmac::new_from_slice(&self.chain_code)
            .expect("chain code should be small enough in size");
        let mut chain_code_hmac: Hmac<Sha512> = hmac::Hmac::new_from_slice(&self.chain_code)
            .expect("chain code should be small enough in size");

        key_hmac.update(&[0u8]);
        key_hmac.update(&self.key);
        key_hmac.update(&self.hash_prefix);
        chain_code_hmac.update(&[1u8]);
        chain_code_hmac.update(&self.key);
        chain_code_hmac.update(&self.hash_prefix);

        key_hmac.update(&index.0.to_le_bytes());
        chain_code_hmac.update(&index.0.to_le_bytes());

        let new_skey: [u8; 64] = key_hmac.finalize_fixed().into();
        let [mut child_skey_left, mut child_skey_right]: [[u8; 32]; 2] = transmute!(new_skey);
        let mut pair_iter = child_skey_left.iter_mut().zip(&self.key);

        let mut carry: u16 = 0;
        pair_iter
            .by_ref()
            .take(28)
            .for_each(|(new_skey_byte, old_skey_byte)| {
                let r = *old_skey_byte as u16 + ((*new_skey_byte as u16) << 3) + carry;
                *new_skey_byte = (r & 0xff) as u8;
                carry = r >> 8;
            });
        pair_iter
            .by_ref()
            .for_each(|(new_skey_byte, old_skey_byte)| {
                let r = *old_skey_byte as u16 + carry;
                *new_skey_byte = (r & 0xff) as u8;
                carry = r >> 8;
            });

        carry = 0;
        child_skey_right
            .iter_mut()
            .zip(self.hash_prefix)
            .for_each(|(new_skey_byte, hp_byte)| {
                let r = *new_skey_byte as u16 + hp_byte as u16 + carry;
                *new_skey_byte = r as u8;
                carry = r >> 8;
            });

        // note: we don't perform the check for curve order divisibility because it will not happen:
        // 1. all keys are in the range K=2^254 .. 2^255 (actually the even smaller range 2^254+2^253)
        // 2. all keys are also multiple of 8
        // 3. all existing multiple of the curve order n in the range of K are not multiple of 8
        // from: https://github.com/typed-io/rust-ed25519-bip32/blob/5a14675ecf9c2940054d984943d71a23e604fcfd/src/derivation/mod.rs#L89C1-L92C93
        let chain_code_hash: [u8; 64] = chain_code_hmac.finalize_fixed().into();
        let [_, child_chain_code]: [[u8; 32]; 2] = transmute!(chain_code_hash);

        Self {
            key: child_skey_left,
            hash_prefix: child_skey_right,
            chain_code: child_chain_code,
        }
    }

    pub fn verifying_key(&self) -> ExtendedVerifyingKey {
        let key = EdwardsPoint::mul_base_clamped(self.key).compress();
        ExtendedVerifyingKey {
            key,
            chain_code: self.chain_code,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtendedVerifyingKey {
    // Invariant: Must be a valid EdwardsPoint
    pub key: VerifyingKey,
    pub chain_code: [u8; 32],
}

impl ExtendedVerifyingKey {
    pub fn derive_child(&self, index: SoftIndex) -> Self {
        use digest::{FixedOutput, KeyInit, Update};
        let mut key_hmac: Hmac<Sha512> = hmac::Hmac::new_from_slice(&self.chain_code)
            .expect("chain code should be small enough in size");
        let mut chain_code_hmac: Hmac<Sha512> = hmac::Hmac::new_from_slice(&self.chain_code)
            .expect("chain code should be small enough in size");

        key_hmac.update(&[2u8]);
        key_hmac.update(&self.key.0);
        key_hmac.update(&index.0.to_le_bytes());
        chain_code_hmac.update(&[3u8]);
        chain_code_hmac.update(&self.key.0);
        chain_code_hmac.update(&index.0.to_le_bytes());

        let z: [u8; 64] = key_hmac.finalize_fixed().into();
        let [mut z_left, _]: [[u8; 32]; 2] = transmute!(z);
        let mut carry = 0;
        z_left[0..28].iter_mut().for_each(|elem| {
            let new = ((*elem as u16) << 3) + carry;
            *elem = new as u8;
            carry = new >> 8;
        });
        z_left[28] = carry as u8;
        z_left[29..].iter_mut().for_each(|elem| {
            *elem = 0;
        });

        let child_key = self.key.decompress().expect("public key should be valid")
            + EdwardsPoint::mul_base(&Scalar::from_bytes_mod_order(z_left));
        let chain_code_hash: [u8; 64] = chain_code_hmac.finalize_fixed().into();
        let [_, child_chain_code]: [[u8; 32]; 2] = transmute!(chain_code_hash);

        Self {
            key: child_key.compress(),
            chain_code: child_chain_code,
        }
    }

    pub fn key(&self) -> ed25519_dalek::VerifyingKey {
        ed25519_dalek::VerifyingKey::from(
            self.key.decompress().expect("public key should be valid"),
        )
    }
}

impl<C> Encode<C> for ExtendedVerifyingKey {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        // Hack because we do not want to allocate to encode two disjoint byte slices into a single
        // CBOR byte string.
        const BYTES_TAG: u8 = 0x40;
        const LEN: u8 = 64;
        let writer = e.writer_mut();
        // Write the CBOR tag for a byte string of length 64.
        writer
            .write_all(&[BYTES_TAG | 24, LEN])
            .map_err(encode::Error::write)?;
        writer
            .write_all(&self.key.0)
            .map_err(encode::Error::write)?;
        writer
            .write_all(&self.chain_code)
            .map_err(encode::Error::write)?;
        Ok(())
    }
}

impl<C> Decode<'_, C> for ExtendedVerifyingKey {
    fn decode(d: &mut Decoder<'_>, _: &mut C) -> Result<Self, decode::Error> {
        let bytes: [u8; 64] = d.bytes()?.try_into().map_err(decode::Error::custom)?;
        let [key, chain_code]: [[u8; 32]; 2] = transmute!(bytes);
        Ok(ExtendedVerifyingKey {
            key: CompressedEdwardsY(key),
            chain_code,
        })
    }
}

#[cfg(test)]
mod tests {
    use ed25519_bip32::XPrv;
    use rand::random;
    use zerocopy::transmute;

    use crate::{ExtendedSecretKey, HardIndex, SoftIndex};

    const D1: [u8; 96] = [
        0xf8, 0xa2, 0x92, 0x31, 0xee, 0x38, 0xd6, 0xc5, 0xbf, 0x71, 0x5d, 0x5b, 0xac, 0x21, 0xc7,
        0x50, 0x57, 0x7a, 0xa3, 0x79, 0x8b, 0x22, 0xd7, 0x9d, 0x65, 0xbf, 0x97, 0xd6, 0xfa, 0xde,
        0xa1, 0x5a, 0xdc, 0xd1, 0xee, 0x1a, 0xbd, 0xf7, 0x8b, 0xd4, 0xbe, 0x64, 0x73, 0x1a, 0x12,
        0xde, 0xb9, 0x4d, 0x36, 0x71, 0x78, 0x41, 0x12, 0xeb, 0x6f, 0x36, 0x4b, 0x87, 0x18, 0x51,
        0xfd, 0x1c, 0x9a, 0x24, 0x73, 0x84, 0xdb, 0x9a, 0xd6, 0x00, 0x3b, 0xbd, 0x08, 0xb3, 0xb1,
        0xdd, 0xc0, 0xd0, 0x7a, 0x59, 0x72, 0x93, 0xff, 0x85, 0xe9, 0x61, 0xbf, 0x25, 0x2b, 0x33,
        0x12, 0x62, 0xed, 0xdf, 0xad, 0x0d,
    ];

    const D1_H0: [u8; 96] = [
        0x60, 0xd3, 0x99, 0xda, 0x83, 0xef, 0x80, 0xd8, 0xd4, 0xf8, 0xd2, 0x23, 0x23, 0x9e, 0xfd,
        0xc2, 0xb8, 0xfe, 0xf3, 0x87, 0xe1, 0xb5, 0x21, 0x91, 0x37, 0xff, 0xb4, 0xe8, 0xfb, 0xde,
        0xa1, 0x5a, 0xdc, 0x93, 0x66, 0xb7, 0xd0, 0x03, 0xaf, 0x37, 0xc1, 0x13, 0x96, 0xde, 0x9a,
        0x83, 0x73, 0x4e, 0x30, 0xe0, 0x5e, 0x85, 0x1e, 0xfa, 0x32, 0x74, 0x5c, 0x9c, 0xd7, 0xb4,
        0x27, 0x12, 0xc8, 0x90, 0x60, 0x87, 0x63, 0x77, 0x0e, 0xdd, 0xf7, 0x72, 0x48, 0xab, 0x65,
        0x29, 0x84, 0xb2, 0x1b, 0x84, 0x97, 0x60, 0xd1, 0xda, 0x74, 0xa6, 0xf5, 0xbd, 0x63, 0x3c,
        0xe4, 0x1a, 0xdc, 0xee, 0xf0, 0x7a,
    ];

    #[test]
    fn derive() {
        let [scalar, hash_prefix, chain_code]: [[u8; 32]; 3] = transmute!(D1);

        eprintln!("{:?}", scalar);
        let skey = ExtendedSecretKey {
            chain_code,
            key: scalar,
            hash_prefix,
        };
        let child = skey.derive_child(HardIndex::new(0x80000000));
        let [expected_scalar, expected_hash_prefix, expected_chain_code]: [[u8; 32]; 3] =
            transmute!(D1_H0);

        assert_eq!(child.key, expected_scalar);
        assert_eq!(child.hash_prefix, expected_hash_prefix);
        assert_eq!(child.chain_code, expected_chain_code);
    }

    #[test]
    fn reference_xprv_derivation() {
        for _ in 0..5 {
            let secret: [u8; 32] = random();
            let cc: [u8; 32] = random();
            let implementation = ExtendedSecretKey::from_nonextended(secret, cc);
            let reference = XPrv::from_nonextended_force(&secret, &cc);
            for _ in 0..5 {
                let index = random::<u32>() | (1 << 31);
                let impl_child = implementation.derive_child(HardIndex::new(index));
                let ref_child = reference.derive(ed25519_bip32::DerivationScheme::V2, index);
                let [ref_key, ref_hash_prefix]: [[u8; 32]; 2] =
                    transmute!(ref_child.extended_secret_key());

                assert_eq!(&impl_child.chain_code, ref_child.chain_code());
                assert_eq!(impl_child.key, ref_key);
                assert_eq!(impl_child.hash_prefix, ref_hash_prefix);
            }
        }
    }

    #[test]
    fn reference_xpub_derivation() {
        for _ in 0..5 {
            let secret: [u8; 32] = random();
            let cc: [u8; 32] = random();
            let implementation = ExtendedSecretKey::from_nonextended(secret, cc);
            let reference = XPrv::from_nonextended_force(&secret, &cc);
            let impl_pub = implementation.verifying_key();
            let ref_pub = reference.public();

            assert_eq!(&impl_pub.chain_code, ref_pub.chain_code());
            assert_eq!(&impl_pub.key.0, ref_pub.public_key_bytes());

            for _ in 0..5 {
                let index = random::<u32>() >> 1;

                let impl_child = impl_pub.derive_child(SoftIndex::new(index));
                let ref_child = ref_pub
                    .derive(ed25519_bip32::DerivationScheme::V2, index)
                    .unwrap();

                assert_eq!(&impl_child.chain_code, ref_child.chain_code());
                assert_eq!(impl_child.key.as_bytes(), ref_child.public_key_bytes());
            }
        }
    }
}
