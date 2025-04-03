use std::time::Duration;

use crate::{traits::state::{AgencyDone, Client, Server, State}, typefu::coproduct::CNil};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Idle;

impl State for Idle {
    const TIMEOUT: Duration = Duration::MAX;

    type Agency = Client;
    type Message = ();
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Busy;

impl State for Busy {
    const TIMEOUT: Duration = Duration::from_secs(60);

    type Agency = Server;
    type Message = ();
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Streaming;

impl State for Streaming {
    const TIMEOUT: Duration = Duration::from_secs(60);
    
    type Agency = Server;
    type Message = ();
}
