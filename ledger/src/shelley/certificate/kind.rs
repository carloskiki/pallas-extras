use crate::{governance, Credential};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DelegateRepresentative {
    Register {
        deposit: u64,
        anchor: Option<governance::Anchor>,
    },
    Unregister {
        deposit: u64,
    },
    Update {
        anchor: Option<governance::Anchor>
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConstitutionalCommittee {
   Authorize(Credential),
   Resign(governance::Anchor)
}
