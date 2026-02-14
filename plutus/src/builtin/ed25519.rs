use ed25519_dalek::{Signature, Verifier, VerifyingKey};

pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> Option<bool> {
    let array_bytes: [u8; 32] = public_key.try_into().ok()?;
    let Ok(public_key) = VerifyingKey::from_bytes(&array_bytes) else {
        return Some(false);
    };
    let signature = Signature::from_slice(signature).ok()?;
    Some(public_key.verify(message, &signature).is_ok())
}
