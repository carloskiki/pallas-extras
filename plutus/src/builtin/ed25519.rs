use ed25519_dalek::{VerifyingKey, Verifier, Signature};

pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> Option<bool> {
    let public_key = VerifyingKey::from_bytes(public_key.try_into().ok()?).ok()?;
    let signature = Signature::from_slice(signature).ok()?;
    Some(public_key.verify(&message, &signature).is_ok())
}
