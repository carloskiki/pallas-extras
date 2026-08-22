pub use ledger::{
    block,
    byron::{
        protocol::{self, Parameters, Version},
        update::{self, Proposal, Vote, proposal::Id},
    },
    crypto::Blake2b224Digest,
    epoch, slot,
};
use ledger::{
    byron::crypto,
    crypto::{
        digest::{FixedOutput, Update as _},
        ed25519_dalek::{self, ed25519::signature::MultipartVerifier},
    },
};
use std::collections::{HashMap, HashSet};
use tinycbor::{Any, Decoder, Encode, Encoder, Memo};
use zerocopy::IntoBytes;

const LOVELACE_PORTION_DENOMINATOR: u128 = 1_000_000_000_000_000;

/// The active delegation relation, indexed by delegate and containing the genesis key that
/// delegated to it.
pub type DelegationMap = HashMap<Blake2b224Digest, Blake2b224Digest>;

pub struct Environment {
    /// The original CBOR encoding of the protocol magic identifier.
    pub protocol_magic: [u8; 5],
    pub security_parameter: block::Number,
    pub slot: slot::Number,
    pub number_of_genesis_keys: u8,
    pub delegation_map: DelegationMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub current_epoch: epoch::Number,
    pub adopted_protocol_version: Version,
    pub adopted_protocol_parameters: Parameters,
    pub candidate_protocol_updates: Vec<CandidateProtocolUpdate>,
    pub application_versions: HashMap<String, ApplicationVersion>,
    pub registered_protocol_update_proposals: HashMap<Id, ProtocolUpdateProposal>,
    pub registered_software_update_proposals: HashMap<Id, SoftwareUpdateProposal>,
    pub confirmed_proposals: HashMap<Id, slot::Number>,
    pub proposal_votes: HashMap<Id, HashSet<Blake2b224Digest>>,
    pub registered_endorsements: HashSet<Endorsement>,
    pub proposal_registration_slots: HashMap<Id, slot::Number>,
}

impl State {
    pub fn new(adopted_protocol_parameters: Parameters) -> Self {
        Self {
            current_epoch: 0,
            adopted_protocol_version: Version {
                major: 0,
                minor: 0,
                patch: 0,
            },
            adopted_protocol_parameters,
            candidate_protocol_updates: Vec::new(),
            application_versions: HashMap::new(),
            registered_protocol_update_proposals: HashMap::new(),
            registered_software_update_proposals: HashMap::new(),
            confirmed_proposals: HashMap::new(),
            proposal_votes: HashMap::new(),
            registered_endorsements: HashSet::new(),
            proposal_registration_slots: HashMap::new(),
        }
    }
}

pub struct Signal<'a> {
    pub update: &'a update::Update<'a>,
    pub endorsement: Endorsement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Endorsement {
    pub protocol_version: Version,
    pub key_hash: Blake2b224Digest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateProtocolUpdate {
    pub slot: slot::Number,
    pub protocol_version: Version,
    pub protocol_parameters: Parameters,
}

/// Metadata is retained in its canonical CBOR representation. The hashes inside Byron update data
/// are opaque to validation, but the metadata must remain available when a software version is
/// confirmed.
pub type Metadata = HashMap<String, Vec<u8>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationVersion {
    pub number: u32,
    pub slot: slot::Number,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolUpdateProposal {
    pub protocol_version: Version,
    pub protocol_parameters: Parameters,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftwareVersion {
    pub application_name: String,
    pub number: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftwareUpdateProposal {
    pub software_version: SoftwareVersion,
    pub metadata: Metadata,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("update proposal registration failed")]
    Registration(#[from] RegistrationError),
    #[error("update vote registration failed")]
    Voting(#[from] VotingError),
    #[error("update endorsement registration failed")]
    Endorsement(#[from] EndorsementError),
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    #[error("a proposal for protocol version {0:?} is already registered")]
    DuplicateProtocolVersion(Version),
    #[error("a proposal for software version {0:?} is already registered")]
    DuplicateSoftwareVersion(SoftwareVersion),
    #[error("the update proposer is not a delegate of a genesis key")]
    InvalidProposer,
    #[error("protocol version {proposed:?} cannot follow adopted version {adopted:?}")]
    InvalidProtocolVersion { proposed: Version, adopted: Version },
    #[error("script version {proposed} cannot follow adopted script version {adopted}")]
    InvalidScriptVersion { adopted: u16, proposed: u16 },
    #[error("the update proposal signature is invalid")]
    InvalidSignature(#[source] ed25519_dalek::SignatureError),
    #[error("the memoized update proposal encoding is invalid")]
    InvalidProposalEncoding,
    #[error("software version {proposed:?} cannot follow the adopted application version")]
    InvalidSoftwareVersion { proposed: SoftwareVersion },
    #[error("maximum block size {proposed} is more than twice the adopted value {adopted}")]
    MaxBlockSizeTooLarge { adopted: u64, proposed: u64 },
    #[error("maximum transaction size {transaction} is not below block size {block}")]
    MaxTransactionSizeTooLarge { block: u64, transaction: u64 },
    #[error("update proposal size {actual} exceeds maximum {maximum}")]
    ProposalTooLarge { maximum: u64, actual: usize },
    #[error("application name is longer than 12 characters: {0}")]
    ApplicationNameTooLong(String),
    #[error("application name contains non-ASCII characters: {0}")]
    ApplicationNameNotAscii(String),
    #[error("system tag is longer than 10 characters: {0}")]
    SystemTagTooLong(String),
    #[error("system tag contains non-ASCII characters: {0}")]
    SystemTagNotAscii(String),
    #[error("the proposal changes neither the protocol nor the software version")]
    NullUpdateProposal,
}

#[derive(Debug, thiserror::Error)]
pub enum VotingError {
    #[error("the update vote signature is invalid")]
    InvalidSignature(#[source] ed25519_dalek::SignatureError),
    #[error("the update vote refers to an unregistered proposal")]
    ProposalNotRegistered,
    #[error("the update voter is not a delegate of a genesis key")]
    VoterNotDelegate,
    #[error("the genesis key has already voted for this proposal")]
    VoteAlreadyCast,
}

#[derive(Debug, thiserror::Error)]
pub enum EndorsementError {
    #[error("multiple proposals target protocol version {0:?}")]
    MultipleProposalsForProtocolVersion(Version),
}

/// Register an update proposal, its votes, and the block issuer's endorsement.
///
/// This is the Byron `BUPI` transition. The transition is atomic: if any subordinate validation
/// rule fails, `state` is left unchanged.
pub fn transition(
    state: &mut State,
    env: &Environment,
    Signal {
        update,
        endorsement,
    }: &Signal<'_>,
) -> Result<(), Error> {
    let mut next = state.clone();

    if let Some(proposal) = &update.proposal {
        register_proposal(&mut next, env, proposal)?;
    }
    register_votes(&mut next, env, &update.votes)?;
    register_endorsement(&mut next, env, *endorsement)?;

    *state = next;
    Ok(())
}

fn register_proposal(
    state: &mut State,
    env: &Environment,
    encoded_proposal: &Memo<'_, Proposal<'_>>,
) -> Result<(), RegistrationError> {
    let proposal = AsRef::<Proposal<'_>>::as_ref(encoded_proposal);
    let proposal_bytes = AsRef::<[u8]>::as_ref(encoded_proposal);
    let proposer = crypto::hash(proposal.issuer.as_bytes());

    if !env.delegation_map.contains_key(&proposer) {
        return Err(RegistrationError::InvalidProposer);
    }
    verify_proposal_signature(env, proposal, proposal_bytes)?;

    let proposal_id = hash_proposal(proposal_bytes);
    let proposed_parameters =
        apply_parameter_update(&state.adopted_protocol_parameters, &proposal.modifications);
    let protocol_version_changed = proposal.protocol_version != state.adopted_protocol_version
        || proposed_parameters != state.adopted_protocol_parameters;
    let software_version_changed = state
        .application_versions
        .get(proposal.software_version.name)
        .is_none_or(|version| version.number != proposal.software_version.version);

    if !protocol_version_changed && !software_version_changed && !is_null_update_exception(env) {
        return Err(RegistrationError::NullUpdateProposal);
    }

    if protocol_version_changed {
        register_protocol_update(
            state,
            proposal,
            proposal_bytes.len(),
            proposal_id,
            proposed_parameters,
        )?;
    }
    if software_version_changed {
        register_software_update(state, proposal, proposal_id)?;
    }

    state
        .proposal_registration_slots
        .insert(proposal_id, env.slot);
    Ok(())
}

fn verify_proposal_signature(
    env: &Environment,
    proposal: &Proposal<'_>,
    proposal_bytes: &[u8],
) -> Result<(), RegistrationError> {
    let signed_fields = proposal_signed_fields(proposal_bytes)?;
    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&proposal.issuer.key)
        .map_err(RegistrationError::InvalidSignature)?;

    let mut message: Vec<&[u8]> = Vec::with_capacity(3 + signed_fields.len());
    message.push(&[0x04]);
    message.push(&env.protocol_magic);
    // The signed value is the five-field proposal body, not the seven-field proposal.
    message.push(&[0x85]);
    message.extend(signed_fields.iter().map(AsRef::as_ref));
    verifying_key
        .multipart_verify(&message, proposal.signature)
        .map_err(RegistrationError::InvalidSignature)
}

fn proposal_signed_fields<'a>(proposal_bytes: &'a [u8]) -> Result<Vec<Any<'a>>, RegistrationError> {
    let mut decoder = Decoder(proposal_bytes);
    let mut proposal = decoder
        .array_visitor()
        .map_err(|_| RegistrationError::InvalidProposalEncoding)?;

    if proposal.remaining() != Some(7) {
        return Err(RegistrationError::InvalidProposalEncoding);
    }

    (0..5)
        .map(|_| {
            proposal
                .visit::<Any<'a>>()
                .ok_or(RegistrationError::InvalidProposalEncoding)?
                .map_err(|_| RegistrationError::InvalidProposalEncoding)
        })
        .collect()
}

fn register_protocol_update(
    state: &mut State,
    proposal: &Proposal<'_>,
    proposal_size: usize,
    proposal_id: Id,
    proposed_parameters: Parameters,
) -> Result<(), RegistrationError> {
    if state
        .registered_protocol_update_proposals
        .values()
        .any(|registered| registered.protocol_version == proposal.protocol_version)
    {
        return Err(RegistrationError::DuplicateProtocolVersion(
            proposal.protocol_version,
        ));
    }
    if !protocol_version_can_follow(proposal.protocol_version, state.adopted_protocol_version) {
        return Err(RegistrationError::InvalidProtocolVersion {
            proposed: proposal.protocol_version,
            adopted: state.adopted_protocol_version,
        });
    }

    validate_parameter_update(
        &state.adopted_protocol_parameters,
        &proposed_parameters,
        proposal_size,
    )?;
    state.registered_protocol_update_proposals.insert(
        proposal_id,
        ProtocolUpdateProposal {
            protocol_version: proposal.protocol_version,
            protocol_parameters: proposed_parameters,
        },
    );
    Ok(())
}

fn protocol_version_can_follow(proposed: Version, adopted: Version) -> bool {
    if proposed <= adopted {
        return false;
    }

    (proposed.major == adopted.major
        && adopted
            .minor
            .checked_add(1)
            .is_some_and(|minor| proposed.minor == minor))
        || (adopted
            .major
            .checked_add(1)
            .is_some_and(|major| proposed.major == major)
            && proposed.minor == 0)
}

fn validate_parameter_update(
    adopted: &Parameters,
    proposed: &Parameters,
    proposal_size: usize,
) -> Result<(), RegistrationError> {
    if proposal_size as u128 > adopted.max_proposal_size as u128 {
        return Err(RegistrationError::ProposalTooLarge {
            maximum: adopted.max_proposal_size,
            actual: proposal_size,
        });
    }
    if adopted
        .max_block_size
        .checked_mul(2)
        .is_some_and(|maximum| proposed.max_block_size > maximum)
    {
        return Err(RegistrationError::MaxBlockSizeTooLarge {
            adopted: adopted.max_block_size,
            proposed: proposed.max_block_size,
        });
    }
    if proposed.max_transaction_size >= proposed.max_block_size {
        return Err(RegistrationError::MaxTransactionSizeTooLarge {
            block: proposed.max_block_size,
            transaction: proposed.max_transaction_size,
        });
    }
    if proposed.script_version != adopted.script_version
        && proposed.script_version != adopted.script_version.wrapping_add(1)
    {
        return Err(RegistrationError::InvalidScriptVersion {
            adopted: adopted.script_version,
            proposed: proposed.script_version,
        });
    }
    Ok(())
}

fn register_software_update(
    state: &mut State,
    proposal: &Proposal<'_>,
    proposal_id: Id,
) -> Result<(), RegistrationError> {
    for (tag, _) in &proposal.data {
        validate_system_tag(tag)?;
    }

    let software_version = SoftwareVersion {
        application_name: proposal.software_version.name.to_owned(),
        number: proposal.software_version.version,
    };
    if state
        .registered_software_update_proposals
        .values()
        .any(|registered| {
            registered.software_version.application_name == software_version.application_name
        })
    {
        return Err(RegistrationError::DuplicateSoftwareVersion(
            software_version,
        ));
    }

    validate_application_name(&software_version.application_name)?;
    let can_follow = match state
        .application_versions
        .get(&software_version.application_name)
    {
        None => matches!(software_version.number, 0 | 1),
        Some(current) => software_version.number == current.number.wrapping_add(1),
    };
    if !can_follow {
        return Err(RegistrationError::InvalidSoftwareVersion {
            proposed: software_version,
        });
    }

    state.registered_software_update_proposals.insert(
        proposal_id,
        SoftwareUpdateProposal {
            software_version,
            metadata: proposal_metadata(proposal),
        },
    );
    Ok(())
}

fn validate_application_name(name: &str) -> Result<(), RegistrationError> {
    if name.chars().count() > 12 {
        return Err(RegistrationError::ApplicationNameTooLong(name.to_owned()));
    }
    if !name.is_ascii() {
        return Err(RegistrationError::ApplicationNameNotAscii(name.to_owned()));
    }
    Ok(())
}

fn validate_system_tag(tag: &str) -> Result<(), RegistrationError> {
    if tag.chars().count() > 10 {
        return Err(RegistrationError::SystemTagTooLong(tag.to_owned()));
    }
    if !tag.is_ascii() {
        return Err(RegistrationError::SystemTagNotAscii(tag.to_owned()));
    }
    Ok(())
}

fn proposal_metadata(proposal: &Proposal<'_>) -> Metadata {
    proposal
        .data
        .iter()
        .map(|(tag, data)| (tag.to_string(), encode_to_vec(data)))
        .collect()
}

fn register_votes(
    state: &mut State,
    env: &Environment,
    votes: &[Vote<'_>],
) -> Result<(), VotingError> {
    for vote in votes {
        register_vote(state, env, vote)?;
    }

    let confirmed_software_updates: Vec<_> = state
        .registered_software_update_proposals
        .iter()
        .filter(|(proposal_id, _)| state.confirmed_proposals.contains_key(*proposal_id))
        .map(|(proposal_id, proposal)| (*proposal_id, proposal.clone()))
        .collect();
    for (_, proposal) in confirmed_software_updates {
        state.application_versions.insert(
            proposal.software_version.application_name,
            ApplicationVersion {
                number: proposal.software_version.number,
                slot: env.slot,
                metadata: proposal.metadata,
            },
        );
    }
    state
        .registered_software_update_proposals
        .retain(|proposal_id, _| !state.confirmed_proposals.contains_key(proposal_id));
    Ok(())
}

fn register_vote(state: &mut State, env: &Environment, vote: &Vote<'_>) -> Result<(), VotingError> {
    let proposal_id = **AsRef::<&Id>::as_ref(&vote.proposal_id);
    let proposal_id_bytes = AsRef::<[u8]>::as_ref(&vote.proposal_id);
    if !state.proposal_registration_slots.contains_key(&proposal_id) {
        return Err(VotingError::ProposalNotRegistered);
    }

    let voter = crypto::hash(vote.voter.as_bytes());
    let Some(delegator) = env.delegation_map.get(&voter).copied() else {
        return Err(VotingError::VoterNotDelegate);
    };
    if state
        .proposal_votes
        .get(&proposal_id)
        .is_some_and(|voters| voters.contains(&delegator))
    {
        return Err(VotingError::VoteAlreadyCast);
    }

    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&vote.voter.key)
        .map_err(VotingError::InvalidSignature)?;
    verifying_key
        // Votes are always interpreted as positive in the Byron-Shelley bridge.
        .multipart_verify(
            &[
                &[0x06],
                &env.protocol_magic,
                &[0x82],
                proposal_id_bytes,
                &[0xf5],
            ],
            vote.signature,
        )
        .map_err(VotingError::InvalidSignature)?;

    let voters = state.proposal_votes.entry(proposal_id).or_default();
    voters.insert(delegator);
    if voters.len() >= adoption_threshold(env, &state.adopted_protocol_parameters) {
        state
            .confirmed_proposals
            .entry(proposal_id)
            .or_insert(env.slot);
    }
    Ok(())
}

fn register_endorsement(
    state: &mut State,
    env: &Environment,
    endorsement: Endorsement,
) -> Result<(), EndorsementError> {
    let matching: Vec<_> = state
        .registered_protocol_update_proposals
        .iter()
        .filter(|(_, proposal)| proposal.protocol_version == endorsement.protocol_version)
        .map(|(proposal_id, proposal)| (*proposal_id, proposal.clone()))
        .collect();

    match matching.as_slice() {
        [] => {}
        [(proposal_id, proposal)] => {
            let stable = state
                .confirmed_proposals
                .get(proposal_id)
                .and_then(|slot| {
                    env.security_parameter
                        .checked_mul(2)
                        .and_then(|stability| slot.checked_add(stability))
                })
                .is_some_and(|stable_at| stable_at <= env.slot);

            if stable {
                if let Some(delegator) = env.delegation_map.get(&endorsement.key_hash) {
                    state.registered_endorsements.insert(Endorsement {
                        protocol_version: endorsement.protocol_version,
                        key_hash: *delegator,
                    });
                }

                let number_of_endorsements = state
                    .registered_endorsements
                    .iter()
                    .filter(|registered| {
                        registered.protocol_version == endorsement.protocol_version
                    })
                    .count();
                if number_of_endorsements
                    >= adoption_threshold(env, &state.adopted_protocol_parameters)
                    && state
                        .candidate_protocol_updates
                        .first()
                        .is_none_or(|candidate| {
                            candidate.protocol_version < endorsement.protocol_version
                        })
                {
                    state.candidate_protocol_updates.insert(
                        0,
                        CandidateProtocolUpdate {
                            slot: env.slot,
                            protocol_version: endorsement.protocol_version,
                            protocol_parameters: proposal.protocol_parameters.clone(),
                        },
                    );
                }
            }
        }
        _ => {
            return Err(EndorsementError::MultipleProposalsForProtocolVersion(
                endorsement.protocol_version,
            ));
        }
    }

    remove_expired_proposals(state, env.slot);
    Ok(())
}

fn remove_expired_proposals(state: &mut State, current_slot: slot::Number) {
    let time_to_live = state.adopted_protocol_parameters.update_proposal_ttl;
    let mut proposals_to_keep: HashSet<_> = state
        .proposal_registration_slots
        .iter()
        .filter(|(_, registered_at)| current_slot <= registered_at.saturating_add(time_to_live))
        .map(|(proposal_id, _)| *proposal_id)
        .collect();
    proposals_to_keep.extend(state.confirmed_proposals.keys().copied());

    state
        .registered_protocol_update_proposals
        .retain(|proposal_id, _| proposals_to_keep.contains(proposal_id));
    state
        .registered_software_update_proposals
        .retain(|proposal_id, _| proposals_to_keep.contains(proposal_id));
    state
        .proposal_votes
        .retain(|proposal_id, _| proposals_to_keep.contains(proposal_id));
    state
        .proposal_registration_slots
        .retain(|proposal_id, _| proposals_to_keep.contains(proposal_id));

    let versions_to_keep: HashSet<_> = state
        .registered_protocol_update_proposals
        .values()
        .map(|proposal| proposal.protocol_version)
        .collect();
    state
        .registered_endorsements
        .retain(|endorsement| versions_to_keep.contains(&endorsement.protocol_version));
}

fn adoption_threshold(env: &Environment, parameters: &Parameters) -> usize {
    ((parameters.soft_fork_rule.minimum_threshold as u128 * env.number_of_genesis_keys as u128)
        / LOVELACE_PORTION_DENOMINATOR) as usize
}

fn apply_parameter_update(
    adopted: &Parameters,
    update: &protocol::parameter::Update,
) -> Parameters {
    Parameters {
        script_version: update
            .script_version()
            .copied()
            .unwrap_or(adopted.script_version),
        slot_duration: update
            .slot_duration()
            .copied()
            .unwrap_or(adopted.slot_duration),
        max_block_size: update
            .max_block_size()
            .copied()
            .unwrap_or(adopted.max_block_size),
        max_header_size: update
            .max_header_size()
            .copied()
            .unwrap_or(adopted.max_header_size),
        max_transaction_size: update
            .max_transaction_size()
            .copied()
            .unwrap_or(adopted.max_transaction_size),
        max_proposal_size: update
            .max_proposal_size()
            .copied()
            .unwrap_or(adopted.max_proposal_size),
        multi_party_computation_threshold: update
            .multi_party_computation_threshold()
            .copied()
            .unwrap_or(adopted.multi_party_computation_threshold),
        heavy_delegation_threshold: update
            .heavy_delegation_threshold()
            .copied()
            .unwrap_or(adopted.heavy_delegation_threshold),
        update_vote_threshold: update
            .update_vote_threshold()
            .copied()
            .unwrap_or(adopted.update_vote_threshold),
        update_proposal_threshold: update
            .update_proposal_threshold()
            .copied()
            .unwrap_or(adopted.update_proposal_threshold),
        update_proposal_ttl: update
            .update_proposal_ttl()
            .copied()
            .unwrap_or(adopted.update_proposal_ttl),
        soft_fork_rule: update
            .soft_fork_rule()
            .copied()
            .unwrap_or(adopted.soft_fork_rule),
        transaction_fee_policy: update
            .transaction_fee_policy()
            .copied()
            .unwrap_or(adopted.transaction_fee_policy),
        unlock_stake_epoch: update
            .unlock_stake_epoch()
            .copied()
            .unwrap_or(adopted.unlock_stake_epoch),
    }
}

fn hash_proposal(bytes: &[u8]) -> Id {
    let mut hasher = ledger::crypto::blake2::Blake2b256::default();
    hasher.update(bytes);
    hasher.finalize_fixed().into()
}

fn encode_to_vec<T: Encode + ?Sized>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    value
        .encode(&mut Encoder(&mut bytes))
        .expect("encoding into a Vec is infallible");
    bytes
}

fn is_null_update_exception(env: &Environment) -> bool {
    env.protocol_magic[0] == 0x1a
        && u32::from_be_bytes(
            env.protocol_magic[1..]
                .try_into()
                .expect("protocol magic has four payload bytes"),
        ) == 633_343_913
        && matches!(env.slot, 969_188 | 1_915_231)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ledger::crypto::ed25519_dalek::ed25519::signature::MultipartSigner;

    fn parameters() -> Parameters {
        Parameters {
            script_version: 0,
            slot_duration: 20_000,
            max_block_size: 2_000_000,
            max_header_size: 2_000,
            max_transaction_size: 1_000_000,
            max_proposal_size: 10_000,
            multi_party_computation_threshold: 0,
            heavy_delegation_threshold: 0,
            update_vote_threshold: 0,
            update_proposal_threshold: 0,
            update_proposal_ttl: 100,
            soft_fork_rule: protocol::soft_fork::Rule {
                initial_threshold: 1_000_000_000_000_000,
                minimum_threshold: 1_000_000_000_000_000,
                decrement: 0,
            },
            transaction_fee_policy: protocol::FeePolicy {
                constant: 0,
                coefficient: 0,
            },
            unlock_stake_epoch: 0,
        }
    }

    fn environment() -> Environment {
        Environment {
            protocol_magic: [0x1a, 0, 0, 0, 1],
            security_parameter: 5,
            slot: 20,
            number_of_genesis_keys: 1,
            delegation_map: HashMap::new(),
        }
    }

    #[test]
    fn protocol_versions_must_increment_major_or_minor() {
        let adopted = Version {
            major: 1,
            minor: 2,
            patch: 3,
        };

        assert!(protocol_version_can_follow(
            Version {
                major: 1,
                minor: 3,
                patch: 0,
            },
            adopted,
        ));
        assert!(protocol_version_can_follow(
            Version {
                major: 2,
                minor: 0,
                patch: 0,
            },
            adopted,
        ));
        assert!(!protocol_version_can_follow(
            Version {
                major: 1,
                minor: 2,
                patch: 4,
            },
            adopted,
        ));
        assert!(!protocol_version_can_follow(
            Version {
                major: 2,
                minor: 1,
                patch: 0,
            },
            adopted,
        ));
    }

    #[test]
    fn parameter_updates_only_replace_present_values() {
        let adopted = parameters();
        let mut update = protocol::parameter::Update::default();
        update.set_script_version(1);
        update.set_max_block_size(3_000_000);

        let applied = apply_parameter_update(&adopted, &update);

        assert_eq!(applied.script_version, 1);
        assert_eq!(applied.max_block_size, 3_000_000);
        assert_eq!(applied.max_transaction_size, adopted.max_transaction_size);
        assert_eq!(applied.soft_fork_rule, adopted.soft_fork_rule);
    }

    #[test]
    fn valid_proposals_are_registered_from_their_memoized_bytes() {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[8; 32]);
        let issuer = ledger::byron::crypto::ExtendedVerifyingKey {
            key: signing_key.verifying_key().to_bytes(),
            chain_code: [1; 32],
        };
        let protocol_version = Version {
            major: 0,
            minor: 0,
            patch: 0,
        };
        let modifications = protocol::parameter::Update::default();
        let software_version = protocol::version::Software {
            name: "cardano",
            version: 0,
        };
        let data: Box<[(&str, update::Data<'_>)]> = Box::new([]);
        let attributes = ledger::byron::Attributes;
        let signed_fields = [
            encode_to_vec(&protocol_version),
            encode_to_vec(&modifications),
            encode_to_vec(&software_version),
            encode_to_vec(&data),
            encode_to_vec(&attributes),
        ];
        let mut env = environment();
        env.delegation_map
            .insert(crypto::hash(issuer.as_bytes()), [2; 28]);
        let mut signed_message: Vec<&[u8]> = vec![&[0x04], &env.protocol_magic, &[0x85]];
        signed_message.extend(signed_fields.iter().map(Vec::as_slice));
        let signature = signing_key.multipart_sign(&signed_message);
        let proposal = Proposal {
            protocol_version,
            modifications,
            software_version,
            data,
            attributes,
            issuer: &issuer,
            signature: &signature,
        };
        let proposal_bytes = encode_to_vec(&proposal);
        let proposal_id = hash_proposal(&proposal_bytes);
        let update = update::Update {
            proposal: Some(Memo::from((proposal, proposal_bytes.as_slice()))),
            votes: Box::new([]),
        };
        let mut state = State::new(parameters());

        transition(
            &mut state,
            &env,
            &Signal {
                update: &update,
                endorsement: Endorsement {
                    protocol_version,
                    key_hash: [0; 28],
                },
            },
        )
        .unwrap();

        assert_eq!(state.proposal_registration_slots[&proposal_id], env.slot);
        assert_eq!(
            state.registered_software_update_proposals[&proposal_id].software_version,
            SoftwareVersion {
                application_name: "cardano".to_owned(),
                number: 0,
            }
        );
    }

    #[test]
    fn votes_are_verified_as_positive_and_confirm_at_the_threshold() {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[7; 32]);
        let voter = ledger::byron::crypto::ExtendedVerifyingKey {
            key: signing_key.verifying_key().to_bytes(),
            chain_code: [0; 32],
        };
        let proposal_id = [9; 32];
        let env = {
            let mut env = environment();
            env.delegation_map
                .insert(crypto::hash(voter.as_bytes()), [3; 28]);
            env
        };
        let signature = signing_key.multipart_sign(&[
            &[0x06],
            &env.protocol_magic,
            &[0x82, 0x58, 0x20],
            &proposal_id,
            &[0xf5],
        ]);
        let mut encoded_proposal_id = [0; 34];
        encoded_proposal_id[..2].copy_from_slice(&[0x58, 0x20]);
        encoded_proposal_id[2..].copy_from_slice(&proposal_id);
        let vote = Vote {
            voter: &voter,
            proposal_id: Memo::from((&proposal_id, encoded_proposal_id.as_slice())),
            // The legacy decision bit is ignored by the bridge rules.
            vote: false,
            signature: &signature,
        };
        let mut state = State::new(parameters());
        state.proposal_registration_slots.insert(proposal_id, 0);

        register_vote(&mut state, &env, &vote).unwrap();

        assert_eq!(state.confirmed_proposals.get(&proposal_id), Some(&env.slot));
        assert_eq!(state.proposal_votes[&proposal_id], HashSet::from([[3; 28]]));
    }

    #[test]
    fn stable_endorsements_create_protocol_candidates() {
        let proposal_id = [4; 32];
        let version = Version {
            major: 0,
            minor: 1,
            patch: 0,
        };
        let delegate = [5; 28];
        let delegator = [6; 28];
        let mut env = environment();
        env.delegation_map.insert(delegate, delegator);
        let mut state = State::new(parameters());
        state.registered_protocol_update_proposals.insert(
            proposal_id,
            ProtocolUpdateProposal {
                protocol_version: version,
                protocol_parameters: parameters(),
            },
        );
        state.confirmed_proposals.insert(proposal_id, 10);
        state.proposal_registration_slots.insert(proposal_id, 0);

        register_endorsement(
            &mut state,
            &env,
            Endorsement {
                protocol_version: version,
                key_hash: delegate,
            },
        )
        .unwrap();

        assert!(state.registered_endorsements.contains(&Endorsement {
            protocol_version: version,
            key_hash: delegator,
        }));
        assert_eq!(state.candidate_protocol_updates.len(), 1);
        assert_eq!(
            state.candidate_protocol_updates[0].protocol_version,
            version
        );
        assert_eq!(state.candidate_protocol_updates[0].slot, env.slot);
    }
}
