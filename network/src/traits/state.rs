use crate::typefu::coproduct::CNil;

pub trait State: Default {
    const TIMEOUT: std::time::Duration;

    type Agency: Agency;
    type Message;
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

#[derive(Default)]
pub struct Done;

impl State for Done {
    const TIMEOUT: std::time::Duration = std::time::Duration::MAX;

    // Final agency goes to the client, so that the `client_send_back` is dropped if the server
    // sends a message that transitions to this state.
    type Agency = Client;
    type Message = CNil;
}
