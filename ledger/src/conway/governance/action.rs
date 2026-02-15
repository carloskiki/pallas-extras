use super::Constitution;
use crate::{
    conway::protocol,
    crypto::Blake2b224Digest,
    epoch, interval,
    shelley::{Credential, address::Account, transaction::Coin},
};
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod id;
pub use id::Id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub enum Action<'a> {
    #[n(0)]
    ParameterChange {
        id: Option<Id<'a>>,
        update: protocol::Parameters,
        policy_hash: Option<&'a Blake2b224Digest>,
    },
    #[n(1)]
    HardForkInitialization {
        id: Option<Id<'a>>,
        version: protocol::Version,
    },
    #[n(2)]
    TreasuryWithdrawals {
        withdrawals: Vec<(Account<'a>, Coin)>,
        policy_hash: Option<&'a Blake2b224Digest>,
    },
    #[n(3)]
    NoConfidence { id: Option<Id<'a>> },
    #[n(4)]
    UpdateCommittee {
        id: Option<Id<'a>>,
        remove: Vec<Credential<'a>>,
        add: Vec<(Credential<'a>, epoch::Number)>,
        signature_threshold: interval::Unit,
    },
    #[n(5)]
    NewConstitution {
        id: Option<Id<'a>>,
        constitution: Constitution<'a>,
    },
    #[n(6)]
    Info,
}
