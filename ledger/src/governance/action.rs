use minicbor::{CborLen, Decode, Encode};

use crate::{
    Credential,
    address::shelley::StakeAddress,
    crypto::Blake2b224Digest,
    epoch,
    protocol::{self, RealNumber},
    transaction::{self, Coin},
};

use super::Constitution;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(flat)]
pub enum Action {
    #[n(0)]
    ParameterChange {
        #[n(0)]
        id: Option<Id>,
        #[n(1)]
        update: protocol::ParameterUpdate,
        #[cbor(n(2), with = "minicbor::bytes")]
        policy_hash: Option<Blake2b224Digest>,
    },
    #[n(1)]
    HardForkInitialization {
        #[n(0)]
        id: Option<Id>,
        #[n(1)]
        version: protocol::Version,
    },
    #[n(2)]
    TreasuryWithdrawals {
        #[cbor(n(0), with = "cbor_util::list_as_map", has_nil)]
        withdrawals: Box<[(StakeAddress, Coin)]>,
        #[cbor(n(1), with = "minicbor::bytes")]
        policy_hash: Option<Blake2b224Digest>,
    },
    #[n(3)]
    NoConfidence {
        #[n(0)]
        id: Option<Id>,
    },
    #[n(4)]
    UpdateCommittee {
        #[n(0)]
        id: Option<Id>,
        #[cbor(n(1), with = "cbor_util::set")]
        remove: Box<[Credential]>,
        #[cbor(n(2), with = "cbor_util::list_as_map")]
        add: Box<[(Credential, epoch::Number)]>,
        #[n(3)]
        signature_threshold: RealNumber,
    },
    #[n(5)]
    NewConstitution {
        #[n(0)]
        id: Option<Id>,
        #[n(1)]
        constitution: Constitution,
    },
    #[n(6)]
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Id {
    #[cbor(n(0), with = "minicbor::bytes")]
    transaction_id: transaction::Id,
    #[n(1)]
    index: u16,
}
