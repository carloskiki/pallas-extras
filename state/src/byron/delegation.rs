pub use ledger::{block, byron::delegation::Certificate, crypto::Blake2b224Digest, epoch, slot};
use ledger::{
    byron::crypto,
    crypto::{
        DigestWriter,
        digest::Update,
        ed25519_dalek::{self, DigestVerifier, Sha512},
    },
};
use ref_cast::RefCast;
use std::collections::{HashMap, HashSet, VecDeque};
use tinycbor::{CborLen, Encode, Encoder};
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
    pub key_epoch_delegations: VecDeque<(epoch::Number, Blake2b224Digest)>,
    pub delegations: HashMap<Blake2b224Digest, (Blake2b224Digest, slot::Number)>,
    pub delegates: HashSet<Blake2b224Digest>,
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

/// Caller must ensure that every call is made with monotonically increasing `slot` and `epoch`
/// environment values.
pub fn transition(
    state: &mut State,
    env: &Environment,
    certificates: &[Certificate<'_>],
) -> Result<(), Error> {
    // 1. Prune any scheduled delegations in previous epochs.
    while state.key_epoch_delegations.pop_front_if(|(epoch, _)| *epoch >= env.epoch).is_some() {}
    
    // 2. Validate and schedule new delegations.
    for certificate in certificates {
        let delegator = crypto::hash(certificate.issuer.as_bytes());
        let delegate = crypto::hash(certificate.delegate.as_bytes());

        let verifying_key = if env.allowed_delegators.contains(&delegator) {
            ed25519_dalek::VerifyingKey::from_bytes(&certificate.issuer.key)?
        } else {
            return Err(Error::UnauthorizedDelegator);
        };
        if !(env.epoch..=env.epoch + 1).contains(&certificate.epoch) {
            return Err(Error::InvalidEpoch);
        }
        if state
            .key_epoch_delegations
            .contains(&(certificate.epoch, delegator))
        {
            return Err(Error::MultipleDelegations);
        }

        verifying_key.verify_digest(
            |d: &mut Sha512| {
                d.update(&[0x0a]);
                d.update(&env.protocol_magic);
                d.update(&[
                    0x58,
                    (2 + certificate.issuer.as_bytes().len() + env.epoch.cbor_len()) as u8,
                ]);
                d.update(b"00");
                d.update(certificate.issuer.as_bytes());
                env.epoch
                    .encode(&mut Encoder(DigestWriter::ref_cast_mut(d)));
                Ok(())
            },
            certificate.signature,
        )?;

        state.scheduled_delegations.push_back(Delegation {
            slot: env.slot + 2 * env.security_parameter,
            delegator,
            delegate,
        });
        
        if certificate.epoch == env.epoch {
            state
                .key_epoch_delegations
                .push_front((certificate.epoch, delegator));
        } else {
            state
                .key_epoch_delegations
                .push_back((certificate.epoch, delegator));
        }
    }

    // 3. Apply any scheduled delegations that are now active.
    while let Some(scheduled_delegation) = state
        .scheduled_delegations
        .pop_front_if(|delegation| delegation.slot <= env.slot)
    {
        if !state.delegates.contains(&scheduled_delegation.delegate)
            && state
                .delegations
                .get(&scheduled_delegation.delegator)
                .is_none_or(|(old_delegate, slot)| {
                    if *slot < scheduled_delegation.slot {
                        state.delegates.remove(old_delegate);
                        true
                    } else {
                        false
                    }
                })
        {
            state.delegates.insert(scheduled_delegation.delegate);
            state.delegations.insert(
                scheduled_delegation.delegator,
                (scheduled_delegation.delegate, scheduled_delegation.slot),
            );
        }
    }

    Ok(())
}
