use super::builtin;
use k256::{
    ecdsa::{
        self,
        signature::{Verifier, hazmat::PrehashVerifier},
    },
    schnorr,
};
use macro_rules_attribute::apply;

#[apply(builtin)]
pub fn verify_ecdsa(public_key: Vec<u8>, message: Vec<u8>, signature: Vec<u8>) -> Option<bool> {
    let public_key = ecdsa::VerifyingKey::from_sec1_bytes(&public_key).ok()?;
    let signature = ecdsa::Signature::from_bytes(&signature.try_into().ok()?).ok()?;
    Some(public_key.verify_prehash(&message, &signature).is_ok())
}

#[apply(builtin)]
pub fn verify_schnorr(public_key: Vec<u8>, message: Vec<u8>, signature: Vec<u8>) -> Option<bool> {
    let public_key = schnorr::VerifyingKey::from_slice(&public_key).ok()?;
    let signature = schnorr::Signature::from_slice(&signature).ok()?;
    Some(public_key.verify(&message, &signature).is_ok())
}
