use ed25519_dalek::Signature;

use crate::{Blake2b224Digest, ExtendedVerifyingKey, VerifyingKey};

pub struct Set {
    pub vkeys: Vec<Vkey>,
    pub scripts: Vec<Script>,
    pub bootstraps: Vec<Bootstrap>,
}

pub struct Vkey {
    pub vkey: VerifyingKey,
    pub signature: Signature,
}

pub enum Script {
    Vkey { addr_hash: Blake2b224Digest },
    All(Vec<Script>),
    Any(Vec<Script>),
    NofK(u64, Vec<Script>),
}

pub struct Bootstrap {
    pub key: ExtendedVerifyingKey,
    pub signature: Signature,
    // TODO: should we ignore the attributes?
    pub attributes: Vec<u8>,
}
