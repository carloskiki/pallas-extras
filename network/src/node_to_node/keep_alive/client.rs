use crate::{message::Done, node_to_node::keep_alive::KeepAlive};

crate::state! {
    Client {
        size_limit: u16::MAX as usize,
        timeout: std::time::Duration::from_secs(97),
        agency: crate::agency::Client,
        message: [KeepAlive, Done<2>]
    }
}

impl crate::state::InitialState for Client {
    const PROTOCOL_ID: u16 = 8;
    const INGRESS_BUFFER_SIZE: usize = 1;
}
