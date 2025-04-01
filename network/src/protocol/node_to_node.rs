pub mod block_fetch;
pub mod chain_sync;

use minicbor::{Decode, Encode};

use crate::traits::protocol::UnknownProtocol;

use super::handshake;

#[derive(Debug, Clone, Copy)]
pub enum NodeToNode {
    Handshake = 0,
    ChainSync = 2,
    BlockFetch = 3,
    TxSubmission = 4,
    KeepAlive = 8,
    PeerSharing = 10,
}

impl From<NodeToNode> for u16 {
    fn from(value: NodeToNode) -> Self {
        value as u16
    }
}

impl TryFrom<u16> for NodeToNode {
    type Error = UnknownProtocol;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NodeToNode::Handshake),
            2 => Ok(NodeToNode::ChainSync),
            3 => Ok(NodeToNode::BlockFetch),
            4 => Ok(NodeToNode::TxSubmission),
            8 => Ok(NodeToNode::KeepAlive),
            10 => Ok(NodeToNode::PeerSharing),
            _ => Err(UnknownProtocol),
        }
    }
}

pub enum NodeToNodeRequest {
    Handshake(handshake::ProposeVersions<handshake::NodeToNodeVersionData>),
    ChainSync(chain_sync::ClientMessage),
}
