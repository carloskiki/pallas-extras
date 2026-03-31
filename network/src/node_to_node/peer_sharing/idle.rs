use crate::{
    agency::Client,
    message::Done,
    node_to_node::peer_sharing::Request,
    state::InitialState,
    state,
};

state! {
    Idle {
        size_limit: 5760,
        timeout: std::time::Duration::MAX,
        agency: Client,
        message: [Request, Done<3>]
    }
}

impl InitialState for Idle {
    const PROTOCOL_ID: u16 = 10;
    const INGRESS_BUFFER_SIZE: usize = 1;
}
