use tinycbor_derive::{CborLen, Decode, Encode};
use crate::shelley::Script;

pub mod bootstrap;
pub use bootstrap::Bootstrap;

pub mod verifying_key;
pub use verifying_key::VerifyingKey;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Set<'a> {
    #[cbor(n(0), optional)]
    pub verifying_keys: Vec<VerifyingKey<'a>>,
    #[cbor(n(1), optional)]
    pub scripts: Vec<Script<'a>>,
    #[cbor(n(2), optional)]
    pub bootstraps: Vec<Bootstrap<'a>>,
}
