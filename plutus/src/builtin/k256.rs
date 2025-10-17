use k256::{ecdsa::{self, signature::{hazmat::PrehashVerifier, Verifier}}, schnorr};

pub fn verify_ecdsa(public_key: &[u8], message: &[u8], signature: &[u8]) -> Option<bool> {
    let public_key = ecdsa::VerifyingKey::from_sec1_bytes(public_key).ok()?;
    let signature = ecdsa::Signature::from_bytes(signature.try_into().ok()?).ok()?;
    Some(public_key.verify_prehash(message, &signature).is_ok())
}

pub fn verify_schnorr(public_key: &[u8], message: &[u8], signature: &[u8]) -> Option<bool> {
    let public_key = schnorr::VerifyingKey::from_slice(public_key).ok()?;
    let signature = schnorr::Signature::from_slice(signature).ok()?;
    Some(public_key.verify(message, &signature).is_ok())
}
