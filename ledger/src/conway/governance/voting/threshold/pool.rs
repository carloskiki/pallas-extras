use tinycbor_derive::{CborLen, Decode, Encode};
use crate::interval;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Pool {
     motion_no_confidence: interval::Unit,
     update_committee: interval::Unit,
     update_committee_no_confidence: interval::Unit,
     hard_fork_initiation: interval::Unit,
     security_protocol_parameter_voting: interval::Unit,
}
