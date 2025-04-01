use crate::traits::protocol::UnknownProtocol;

#[derive(Debug, Clone, Copy)]
pub enum NodeToClient {
    Handshake = 0,
    ChainSync = 5,
    LocalTxSubmission = 6,
    LocalStateQuery = 7,
    LocalTxMonitor = 9,
}

impl From<NodeToClient> for u16 {
    fn from(value: NodeToClient) -> Self {
        value as u16
    }
}

impl TryFrom<u16> for NodeToClient {
    type Error = UnknownProtocol;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NodeToClient::Handshake),
            5 => Ok(NodeToClient::ChainSync),
            6 => Ok(NodeToClient::LocalTxSubmission),
            7 => Ok(NodeToClient::LocalStateQuery),
            9 => Ok(NodeToClient::LocalTxMonitor),
            _ => Err(UnknownProtocol),
        }
    }
}

