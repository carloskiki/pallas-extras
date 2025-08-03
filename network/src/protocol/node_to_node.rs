pub mod block_fetch;
pub mod chain_sync;
pub mod keep_alive;
pub mod peer_sharing;
pub mod tx_submission;

use block_fetch::BlockFetch;
use chain_sync::ChainSync;
use keep_alive::KeepAlive;
use peer_sharing::PeerSharing;
use tx_submission::TxSubmission;

use crate::typefu::coproduct::Coprod;

use super::handshake::{Handshake, message::NodeToNodeVersionData};

/// The node-to-node protocol.
pub type NodeToNode = Coprod![
    Handshake<NodeToNodeVersionData>,
    ChainSync,
    BlockFetch,
    TxSubmission,
    KeepAlive,
    PeerSharing
];
