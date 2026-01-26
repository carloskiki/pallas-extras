use tinycbor_derive::{CborLen, Decode, Encode};
use crate::interval;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct DelegateRepresentative {
    motion_no_confidence: interval::Unit,
    update_committee: interval::Unit,
    update_committee_no_confidence: interval::Unit,
    update_constitution: interval::Unit,
    hard_fork_initiation: interval::Unit,
    protocol_parameter_network_update: interval::Unit,
    protocol_parameter_economic_update: interval::Unit,
    protocol_parameter_technical_update: interval::Unit,
    protocol_parameter_security_update: interval::Unit,
}
