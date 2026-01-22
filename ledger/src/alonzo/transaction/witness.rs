use crate::{allegra, shelley::transaction::witness::{VerifyingKey, Bootstrap}};

pub struct Set<'a> {
    pub verifying_keys: Vec<VerifyingKey<'a>>,
    pub native_scripts: Vec<allegra::Script<'a>>,
    pub bootstraps: Vec<Bootstrap<'a>>,
}
