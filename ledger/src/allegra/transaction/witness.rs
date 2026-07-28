use crate::allegra::Script;
use crate::shelley::transaction::witness::{Bootstrap, VerifyingKey};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Set<'a> {
    #[cbor(n(0), optional)]
    pub verifying_keys: Box<[VerifyingKey<'a>]>,
    #[cbor(n(1), optional)]
    pub scripts: Box<[Script<'a>]>,
    #[cbor(n(2), optional)]
    pub bootstraps: Box<[Bootstrap<'a>]>,
}
