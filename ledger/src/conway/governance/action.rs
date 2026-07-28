use crate::{
    Coin, Unique,
    conway::{governance::Constitution, protocol},
    crypto::Blake2b224Digest,
    epoch, interval,
    shelley::{Credential, address::Account},
    unique,
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
        withdrawals: Unique<Box<[(Account, Coin)]>, false>,
        policy_hash: Option<&'a Blake2b224Digest>,
    },
    #[n(3)]
    NoConfidence { id: Option<Id<'a>> },
    #[n(4)]
    UpdateCommittee {
        id: Option<Id<'a>>,
        #[cbor(decode_with = "unique::codec::Tagged<Credential>")]
        remove: Unique<Box<[Credential]>, false>,
        add: Unique<Box<[(Credential, epoch::Number)]>, false>,
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
