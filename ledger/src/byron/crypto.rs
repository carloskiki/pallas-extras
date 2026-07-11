//! Byron era specific cryptographic types.

use crate::crypto::{Blake2b224, Blake2b224Digest, DigestWriter, digest::Digest};
use sha3::Sha3_256;
use tinycbor::Encode;
use zerocopy::IntoBytes;

pub type ExtendedVerifyingKey = bip32::ExtendedVerifyingKey;

/// Computes the digest of a serialized [`ExtendedVerifyingKey`] according to the Byron era
/// specification.
pub fn key_digest(key: &ExtendedVerifyingKey) -> Blake2b224Digest {
    let mut encoder = tinycbor::Encoder(DigestWriter(Sha3_256::default()));
    key.as_bytes().encode(&mut encoder);
    Blake2b224::digest(encoder.0.0.finalize()).into()
}
