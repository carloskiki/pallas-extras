// TODO: Types for the different hashes and signatures

pub struct HeaderBody {
    pub block_number: u64,
    pub slot_number: u64,
    pub previous_hash: Option<[u8; 32]>,
    pub vrf_vkey: [u8; 32],
    pub vrf_certification: VrfCertificate,
    pub block_body_size: u64,
    pub block_body_hash: [u8; 32],
    pub operational_certificate: OperationalCertificate,
    /// TODO: (Major, Minor). Major is in the range [1 .. 9], so maybe we can use u8 or an enum.
    pub protocol_version: (u64, u64),
}

pub struct OperationalCertificate {
    pub kes_public_key: [u8; 32],
    pub sequence_number: u64,
    pub key_period: u64,
    // TODO
    // pub signature: 
}

// TODO
pub struct VrfCertificate {}
