use tinycbor::Any;
use tinycbor_derive::{CborLen, Decode, Encode};

use crate::{byron::transaction, crypto};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Proof<'a> {
    transaction_proof: transaction::Proof,
    ssc_proof: Any<'a>,
    delegation_proof: crypto::Blake2b256Digest,
    update_proof: crypto::Blake2b256Digest,
}
