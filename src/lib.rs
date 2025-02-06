use curve25519_dalek::{edwards::CompressedEdwardsY, scalar::clamp_integer, EdwardsPoint, Scalar};
use ed25519_dalek::{SecretKey, Sha512};
use hmac::Hmac;
use sha2::{Digest, Sha256};
use zerocopy::transmute;

pub struct ExtendedSecretKey {
    // Invariant: Must be a valid scalar (clamped).
    key: SecretKey,
    pub hash_prefix: [u8; 32],
    pub chain_code: [u8; 32],
}

impl ExtendedSecretKey {
    /// This forces the third bit of the highest byte to a zero, which may not be what you want. If you
    /// want the function to error when the master key is not valid, use `TryFrom` instead.
    pub fn new_force(master: SecretKey) -> Self {
        let full_key: [u8; 64] = Sha512::digest(master).into();
        let [mut key, hash_prefix]: [[u8; 32]; 2] = transmute!(full_key);
        key[31] &= 0b1101_1111;
        let key = clamp_integer(key);
        let chain_code: [u8; 32] = Sha256::new_with_prefix([1])
            .chain_update(master)
            .finalize()
            .into();
        Self {
            key,
            hash_prefix,
            chain_code,
        }
    }

    pub fn derive_child(&self, index: u32) -> Self {
        use digest::{FixedOutput, KeyInit, Update};
        let mut key_hmac: Hmac<Sha512> = hmac::Hmac::new_from_slice(&self.chain_code)
            .expect("chain code should be small enough in size");
        let mut chain_code_hmac: Hmac<Sha512> = hmac::Hmac::new_from_slice(&self.chain_code)
            .expect("chain code should be small enough in size");

        if index < (1 << 31) {
            let point = EdwardsPoint::mul_base_clamped(self.key).compress();
            key_hmac.update(&[2u8]);
            key_hmac.update(point.as_bytes());
            chain_code_hmac.update(&[3u8]);
            chain_code_hmac.update(point.as_bytes());
        } else {
            key_hmac.update(&[0u8]);
            key_hmac.update(&self.key);
            key_hmac.update(&self.hash_prefix);
            chain_code_hmac.update(&[1u8]);
            chain_code_hmac.update(&self.key);
            chain_code_hmac.update(&self.hash_prefix);
        }
        key_hmac.update(&index.to_le_bytes());
        chain_code_hmac.update(&index.to_le_bytes());

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

/// The master provided to derive the extended secret key was invalid.
pub struct InvalidMaster;

impl TryFrom<SecretKey> for ExtendedSecretKey {
    type Error = InvalidMaster;

    fn try_from(master: SecretKey) -> Result<Self, Self::Error> {
        let full_key: [u8; 64] = Sha512::digest(master).into();
        let [mut key, hash_prefix]: [[u8; 32]; 2] = transmute!(full_key);
        key[31] &= 0b1101_1111;
        let key = clamp_integer(key);
        let chain_code: [u8; 32] = Sha256::new_with_prefix([1])
            .chain_update(master)
            .finalize()
            .into();
        Ok(Self {
            key,
            hash_prefix,
            chain_code,
        })
    }
}

pub struct ExtendedVerifyingKey {
    // Invariant: Must be a valid EdwardsPoint
    key: CompressedEdwardsY,
    pub chain_code: [u8; 32],
}

impl ExtendedVerifyingKey {
    /// This truncates the index to the range [0, 2^31), forcing normal derivation (to have
    /// hardened derivation, private key is needed).
    pub fn derive_child(&self, mut index: u32) -> Self {
        index &= (1 << 31) - 1;

        use digest::{FixedOutput, KeyInit, Update};
        let mut key_hmac: Hmac<Sha512> = hmac::Hmac::new_from_slice(&self.chain_code)
            .expect("chain code should be small enough in size");
        let mut chain_code_hmac: Hmac<Sha512> = hmac::Hmac::new_from_slice(&self.chain_code)
            .expect("chain code should be small enough in size");

        key_hmac.update(&[2u8]);
        key_hmac.update(&self.key.0);
        key_hmac.update(&index.to_le_bytes());
        chain_code_hmac.update(&[3u8]);
        chain_code_hmac.update(&self.key.0);
        chain_code_hmac.update(&index.to_le_bytes());

        let z: [u8; 64] = key_hmac.finalize_fixed().into();
        let [mut z_left, _]: [[u8; 32]; 2] = transmute!(z);
        let mut iter = z_left.iter_mut();
        let mut carry = 0;
        iter.by_ref().take(28).for_each(|elem| {
            let new = ((*elem as u16) << 3) + carry;
            *elem = new as u8;
            carry = new >> 8;
        });
        *iter.by_ref().next().unwrap() = carry as u8;
        iter.by_ref().for_each(|elem| {
            *elem = 0;
        });

        let child_key = self.key.decompress().unwrap()
            + EdwardsPoint::mul_base(&Scalar::from_bytes_mod_order(z_left));
        let chain_code_hash: [u8; 64] = chain_code_hmac.finalize_fixed().into();
        let [_, child_chain_code]: [[u8; 32]; 2] = transmute!(chain_code_hash);

        Self {
            key: child_key.compress(),
            chain_code: child_chain_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use ed25519_bip32::XPrv;
    use ed25519_dalek::SecretKey;
    use rand::random;
    use zerocopy::transmute;

    use crate::ExtendedSecretKey;

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
        let child = skey.derive_child(0x80000000);
        let [expected_scalar, expected_hash_prefix, expected_chain_code]: [[u8; 32]; 3] =
            transmute!(D1_H0);

        assert_eq!(child.key, expected_scalar);
        assert_eq!(child.hash_prefix, expected_hash_prefix);
        assert_eq!(child.chain_code, expected_chain_code);
    }

    #[test]
    fn reference_xprv_derivation() {
        for _ in 0..100 {
            let master: SecretKey = random();
            let implementation = ExtendedSecretKey::new_force(master);
            let reference = XPrv::from_nonextended_force(&master, &implementation.chain_code);
            for _ in 0..100 {
                let index: u32 = random();
                let impl_child = implementation.derive_child(index);
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
        for _ in 0..100 {
            let master: SecretKey = random();
            let implementation = ExtendedSecretKey::new_force(master);
            let reference = XPrv::from_nonextended_force(&master, &implementation.chain_code);
            let impl_pub = implementation.verifying_key();
            let ref_pub = reference.public();

            assert_eq!(&impl_pub.chain_code, ref_pub.chain_code());
            assert_eq!(&impl_pub.key.0, ref_pub.public_key_bytes());

            for _ in 0..100 {
                let index = random::<u32>() >> 1;

                let impl_child = impl_pub.derive_child(index);
                let ref_child = ref_pub
                    .derive(ed25519_bip32::DerivationScheme::V2, index)
                    .unwrap();

                assert_eq!(&impl_child.chain_code, ref_child.chain_code());
                assert_eq!(impl_child.key.as_bytes(), ref_child.public_key_bytes());
            }
        }
    }
}
