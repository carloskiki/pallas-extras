pub use ledger::{block, byron::delegation::Certificate, crypto::Blake2b224Digest, epoch, slot};
use ledger::{byron::crypto, crypto::ed25519_dalek::{self, ed25519::signature::MultipartVerifier}};
use std::collections::{HashSet, VecDeque};
use tinycbor::{Encode, Encoder};
use zerocopy::IntoBytes;

pub struct Environment {
    pub protocol_magic: [u8; 5],
    pub allowed_delegators: HashSet<Blake2b224Digest>,
    pub epoch: epoch::Number,
    pub slot: slot::Number,
    pub security_parameter: block::Number,
}

pub struct State {
    pub scheduled_delegations: VecDeque<Delegation>,
    pub key_epoch_delegations: HashSet<(epoch::Number, Blake2b224Digest)>,
}

pub struct Delegation {
    pub slot: slot::Number,
    pub delegator: Blake2b224Digest,
    pub delegate: Blake2b224Digest,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("delegator is not allowed to delegate")]
    UnauthorizedDelegator,
    #[error("delegator (public key, signature) pair is invalid")]
    InvalidCryptography(#[from] ed25519_dalek::SignatureError),
    #[error("certificate epoch is invalid for current epoch")]
    InvalidEpoch,
    #[error("delegator has already delegated in this epoch")]
    MultipleDelegations,
}

pub fn transition(
    state: &mut State,
    env: &Environment,
    certificate: &Certificate<'_>,
) -> Result<(), Error> {
    let delegator_hash = crypto::hash(certificate.issuer.as_bytes());
    let _delegate_hash = crypto::hash(certificate.delegate.as_bytes());

    let verifying_key = if env.allowed_delegators.contains(&delegator_hash) {
        ed25519_dalek::VerifyingKey::from_bytes(&certificate.issuer.key)?
    } else {
        return Err(Error::UnauthorizedDelegator);
    };
    if !(env.epoch..=env.epoch + 1).contains(&certificate.epoch) {
        return Err(Error::InvalidEpoch);
    }

    if state
        .key_epoch_delegations
        .contains(&(certificate.epoch, delegator_hash))
    {
        return Err(Error::MultipleDelegations);
    }

    let mut stream_verifier = verifying_key.verify_stream(certificate.signature)?;

    todo!();
}
