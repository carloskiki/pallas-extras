use tinycbor_derive::{CborLen, Decode, Encode};

use crate::{byron::transaction, crypto};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Proof {
    transaction_proof: transaction::Proof,
    delegation_proof: crypto::Blake2b256Digest,
    update_proof: crypto::Blake2b256Digest,
}
