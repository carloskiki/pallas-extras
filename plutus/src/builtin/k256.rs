use k256::{
    ecdsa::{self, signature::hazmat::PrehashVerifier},
    schnorr,
};

pub fn verify_ecdsa(verifying_key: &[u8], message: &[u8], signature: &[u8]) -> Option<bool> {
    if message.len() != 32 {
        return None;
    }

    let verifying_key = ecdsa::VerifyingKey::from_sec1_bytes(verifying_key).ok()?;
    let signature = ecdsa::Signature::from_bytes(signature.try_into().ok()?).ok()?;
    Some(verifying_key.verify_prehash(message, &signature).is_ok())
}

pub fn verify_schnorr(verifying_key: &[u8], message: &[u8], signature: &[u8]) -> Option<bool> {
    let verifying_key = schnorr::VerifyingKey::from_slice(verifying_key).ok()?;
    let Ok(signature) = schnorr::Signature::from_bytes(signature.try_into().ok()?) else {
        return Some(false);
    };
    Some(verifying_key.verify_prehash(message, &signature).is_ok())
}
