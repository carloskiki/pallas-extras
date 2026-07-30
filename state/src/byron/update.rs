use std::collections::{HashMap, HashSet};

use ledger::{byron::{self, Update}, crypto::Blake2b224Digest, slot};

pub enum Error {}

pub struct Environment {
    delegations: HashSet<Blake2b224Digest>,
}

pub fn transition(udpate: &Update, env: &Environment) -> Result<(), Error> {
    // if let Some(proposal) = &udpate.proposal {
    //     let proposer = byron::crypto::hash(proposal.as_ref().issuer.as_bytes());
    // }

    todo!()
}
