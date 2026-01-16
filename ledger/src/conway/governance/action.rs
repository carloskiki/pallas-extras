use tinycbor_derive::{CborLen, Decode, Encode};

use crate::{
    conway::{UnitInterval, governance::Id, protocol, transaction::Coin},
    crypto::Blake2b224Digest,
    epoch,
    shelley::{Credential, address::Account},
};

use super::Constitution;

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
        #[cbor(with = "cbor_util::Set<Credential<'a>, false>")]
        remove: Vec<Credential<'a>>,
        add: Vec<(Credential<'a>, epoch::Number)>,
        signature_threshold: UnitInterval,
    },
    #[n(5)]
    NewConstitution {
        id: Option<Id<'a>>,
        constitution: Constitution<'a>,
    },
    #[n(6)]
    Info,
}
