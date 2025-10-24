use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use macro_rules_attribute::apply;

use super::builtin;

#[apply(builtin)]
pub fn verify(public_key: Vec<u8>, message: Vec<u8>, signature: Vec<u8>) -> Option<bool> {
    let public_key = VerifyingKey::from_bytes(&public_key.try_into().ok()?).ok()?;
    let signature = Signature::from_slice(&signature).ok()?;
    Some(public_key.verify(&message, &signature).is_ok())
}
