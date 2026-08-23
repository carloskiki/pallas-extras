#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

//! Independent asynchronous actors for peer management, chain synchronization, and block fetch.

pub mod block_fetch;
pub mod cardano;
pub mod chain_sync;
pub mod peer_manager;
pub mod simulation;
pub mod transport;
pub mod types;

pub use block_fetch::{BlockFetcher, BlockFetcherConfig, BlockFetcherHandle, FetchReceipt};
pub use cardano::CardanoConnector;
pub use chain_sync::{
    ChainSnapshot, ChainSynchronizer, ChainSynchronizerConfig, ChainSynchronizerHandle,
};
pub use peer_manager::{PeerManager, PeerManagerConfig, PeerManagerHandle, PeerSnapshot};
pub use simulation::FaultPlan;
pub use transport::{BoxFuture, Connector, PeerConnection, TransportError};
pub use types::{Block, ChainEvent, Digest, Header, Point, Tip};

/// Pinned Ouroboros node-to-node protocol version used by the Cardano transport.
pub const NODE_TO_NODE_VERSION: u16 = 14;

/// Reference Cardano Node release used for interoperability evidence.
pub const CARDANO_NODE_VERSION: &str = "11.0.1";
