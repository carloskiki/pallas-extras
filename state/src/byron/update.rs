use std::collections::{HashMap, HashSet};

use ledger::{
    byron::{self, Update},
    crypto::{
        Blake2b224Digest,
        ed25519_dalek::{self, ed25519::signature::MultipartVerifier},
    },
    slot,
};
use tinycbor::CborLen;
use zerocopy::IntoBytes;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("proposer is not certified by a delegator")]
    InvalidProposer,
    #[error("the signature or public key of the proposal is invalid")]
    InvalidCryptography(#[from] ed25519_dalek::SignatureError),
    #[error("the proposal protocol version was not bumped to a higher version while modifying the parameters")]
    StaleVersion,
    #[error("the update does not update anything")]
    EmptyUpdate,
}

pub struct Environment {
    network_magic: [u8; 5],
    delegations: HashSet<Blake2b224Digest>,
    protocol_version: byron::protocol::Version,
    protocol_parameters: byron::protocol::Parameters,
}

pub fn transition(udpate: &Update, env: &Environment) -> Result<(), Error> {
    if let Some(proposal) = &udpate.proposal {
        let Some(proposal_bytes) = proposal.bytes() else {
            unimplemented!("non-memoized proposals are unsupported for byron validation");
        };
        let proposal = proposal.as_ref();
        if !env
            .delegations
            .contains(&byron::crypto::hash(proposal.issuer.as_bytes()))
        {
            return Err(Error::InvalidProposer);
        }
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&proposal.issuer.key)?;

        let tail_cbor_len =
            proposal.issuer.as_bytes().cbor_len() + proposal.signature.as_bytes().cbor_len();
        // We assume cannonical encoding which is valid for Cardano mainnet.
        let signed_bytes = &proposal_bytes[1..proposal_bytes.len() - tail_cbor_len];
        verifying_key.multipart_verify(
            &[&[0x04], &env.network_magic, &[0x85], signed_bytes],
            proposal.signature,
        )?;

        if proposal.protocol_version < env.protocol_version {
            return Err(Error::StaleVersion);
        }
        
    }

    todo!()
}
