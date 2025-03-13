// TODO: Types for the different hashes and signatures

use ed25519_dalek::SigningKey;

use crate::{Blake2b256, Blake2b256Digest};

pub struct HeaderBody {
    pub block_number: u64,
    pub slot: u64,
    pub previous_hash: Option<[u8; 32]>,
    pub issuer_vkey: crate::VerifyingKey,
    pub vrf_vkey: crate::VerifyingKey,
    /// In Babbage and beyond, this serves both as the leader VRF and the nonce VRF.
    pub leader_vrf: VrfCertificate,
    pub nonce_vrf: Option<VrfCertificate>,
    pub block_body_size: u32,
    pub block_body_hash: Blake2b256Digest,
    pub operational_certificate: OperationalCertificate,
    pub protocol_version: (u64, u64),
}

pub struct OperationalCertificate {
    pub kes_public_key: kes::sum::VerifyingKey<SigningKey, SigningKey, Blake2b256>,
    pub sequence_number: u64,
    pub key_period: u64,
    pub signature: ed25519_dalek::Signature,
}

// TODO
pub struct VrfCertificate {
    pub _figure_what_this_is: Vec<u8>,
    // TODO: use the correct proof type once implemented in an upstream crate.
    pub proof: [u8; 80],
}
