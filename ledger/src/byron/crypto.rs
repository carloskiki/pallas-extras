//! Byron era specific cryptographic types.

use crate::crypto::{Blake2b224, digest::{Update, FixedOutput, OutputSizeUser, HashMarker, Reset, FixedOutputReset}};
use sha3::Sha3_256;

pub type ExtendedVerifyingKey = bip32::ExtendedVerifyingKey;

#[derive(Clone, Debug, Default)]
pub struct Sha3_256Blake2b224(pub Sha3_256);

impl Update for Sha3_256Blake2b224 {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }
}

impl OutputSizeUser for Sha3_256Blake2b224 {
    type OutputSize = <Blake2b224 as OutputSizeUser>::OutputSize;
}

impl FixedOutput for Sha3_256Blake2b224 {
    fn finalize_into(self, out: &mut digest::Output<Self>) {
        let tmp = self.0.finalize_fixed();
        let mut digest = Blake2b224::default();
        digest.update(&tmp);
        digest.finalize_into(out);
    }
}

impl Reset for Sha3_256Blake2b224 {
    fn reset(&mut self) {
        self.0.reset();
    }
}

impl FixedOutputReset for Sha3_256Blake2b224 {
    fn finalize_into_reset(&mut self, out: &mut digest::Output<Self>) {
        let tmp = self.0.finalize_fixed_reset();
        let mut digest = Blake2b224::default();
        digest.update(&tmp);
        digest.finalize_into(out);
    }
}

impl HashMarker for Sha3_256Blake2b224 {}

pub fn hash<T: tinycbor::Encode + ?Sized>(data: &T) -> crate::crypto::Blake2b224Digest {
    let mut hasher = crate::crypto::DigestWriter::<Sha3_256Blake2b224>::default();
    data.encode(&mut tinycbor::Encoder(&mut hasher));
    hasher.0.finalize_fixed().into()
}
