use minicbor::{Decode, Encode};

use crate::typefu::{coproduct::CNil, map::TypeMap};

pub trait State: Default {
    const TIMEOUT: std::time::Duration;

    type Agency: Agency;
    type Message: Encode<()> + for<'a> Decode<'a, ()> + 'static;
}

pub trait Agency {
    const SERVER: bool;
}

pub enum Client {}
impl Agency for Client {
    const SERVER: bool = false;
}

pub enum Server {}
impl Agency for Server {
    const SERVER: bool = true;
}

pub enum AgencyDone {}
impl Agency for AgencyDone {
    const SERVER: bool = false;
}

#[derive(Default)]
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
