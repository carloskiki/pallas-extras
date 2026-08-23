use crate::{Block, ChainEvent, Point};
use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::sync::Mutex;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Opens and handshakes a node-to-node connection.
pub trait Connector: Send + Sync + 'static {
    fn connect(&self, peer: SocketAddr) -> BoxFuture<'_, Result<PeerConnection, TransportError>>;
}

pub trait KeepAliveClient: Send {
    fn keep_alive(&mut self) -> BoxFuture<'_, Result<(), TransportError>>;
}

pub trait PeerSharingClient: Send {
    fn share_peers(&mut self, amount: u8)
    -> BoxFuture<'_, Result<Vec<SocketAddr>, TransportError>>;
}

pub trait ChainSyncClient: Send {
    fn find_intersect(
        &mut self,
        points: &[Point],
    ) -> BoxFuture<'_, Result<Option<Point>, TransportError>>;

    fn next(&mut self) -> BoxFuture<'_, Result<ChainEvent, TransportError>>;
}

pub trait BlockFetchClient: Send {
    fn fetch_range(
        &mut self,
        start: Point,
        end: Point,
    ) -> BoxFuture<'_, Result<Vec<Block>, TransportError>>;
}

/// A handshaken peer split into independently lockable mini-protocol clients.
pub struct PeerConnection {
    pub peer: SocketAddr,
    pub keep_alive: Option<Mutex<Box<dyn KeepAliveClient>>>,
    pub peer_sharing: Option<Mutex<Box<dyn PeerSharingClient>>>,
    pub chain_sync: Mutex<Box<dyn ChainSyncClient>>,
    pub block_fetch: Mutex<Box<dyn BlockFetchClient>>,
    healthy: AtomicBool,
}

impl PeerConnection {
    pub fn new(
        peer: SocketAddr,
        keep_alive: Option<Box<dyn KeepAliveClient>>,
        peer_sharing: Option<Box<dyn PeerSharingClient>>,
        chain_sync: Box<dyn ChainSyncClient>,
        block_fetch: Box<dyn BlockFetchClient>,
    ) -> Self {
        Self {
            peer,
            keep_alive: keep_alive.map(Mutex::new),
            peer_sharing: peer_sharing.map(Mutex::new),
            chain_sync: Mutex::new(chain_sync),
            block_fetch: Mutex::new(block_fetch),
            healthy: AtomicBool::new(true),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Relaxed)
    }

    pub fn mark_unhealthy(&self) {
        self.healthy.store(false, Ordering::Relaxed);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TransportError {
    #[error("peer disconnected")]
    Disconnected,
    #[error("mini-protocol message was dropped by the fault injector")]
    Dropped,
    #[error("peer rejected node-to-node handshake: {0}")]
    Handshake(String),
    #[error("malformed node-to-node message: {0}")]
    Malformed(String),
    #[error("transport I/O error: {0}")]
    Io(String),
    #[error("requested range is not available")]
    NoBlocks,
}

impl From<std::io::Error> for TransportError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
