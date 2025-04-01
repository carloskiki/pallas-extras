use crate::typefu::{coproduct::CNil, map::TypeMap};

pub trait State {
    const TIMEOUT: std::time::Duration;

    type Agency: Agency;
    type Message;
}

pub trait Agency {
    const SERVER: Option<bool>;
}

pub enum Client {}
impl Agency for Client {
    const SERVER: Option<bool> = Some(false);
}

pub enum Server {}
impl Agency for Server {
    const SERVER: Option<bool> = Some(true);
}

enum AgencyDone {}
impl Agency for AgencyDone {
    const SERVER: Option<bool> = None;
}

pub struct Done;

impl State for Done {
    const TIMEOUT: std::time::Duration = std::time::Duration::MAX;

    type Agency = AgencyDone;

    type Message = CNil;
}

pub enum Message {}
impl<S> TypeMap<S> for Message
where
    S: State,
{
    type Output = S::Message;
}
