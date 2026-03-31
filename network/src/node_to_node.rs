pub mod block_fetch;
pub mod chain_sync;
pub mod keep_alive;
pub mod peer_sharing;
pub mod tx_submission;

mod version_data;
pub use version_data::VersionData;

/// The node-to-node protocol.
pub type NodeToNode = (
    crate::handshake::Propose<VersionData>,
    chain_sync::Idle,
    block_fetch::Idle,
    tx_submission::Init,
    keep_alive::Client,
    peer_sharing::Idle,
);
