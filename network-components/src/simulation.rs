use crate::{
    Block, ChainEvent, Header, Point, Tip,
    transport::{
        BlockFetchClient, BoxFuture, ChainSyncClient, Connector, KeepAliveClient, PeerConnection,
        PeerSharingClient, TransportError,
    },
};
use std::{
    collections::BTreeSet,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::sync::RwLock;

#[derive(Clone, Debug, Default)]
pub struct FaultPlan {
    /// Drop these one-indexed mini-protocol calls on every connection.
    pub drop_calls: BTreeSet<u64>,
    /// Drop every Nth mini-protocol call. Zero disables periodic drops.
    pub drop_every: u64,
    /// Disconnect after this many calls. Zero keeps the connection alive.
    pub disconnect_after: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SimulationStatistics {
    pub connection_attempts: u64,
    pub protocol_calls: u64,
    pub dropped_messages: u64,
    pub disconnected_sessions: u64,
}

#[derive(Clone)]
pub struct SimulationNetwork {
    inner: Arc<Inner>,
}

struct Inner {
    peers: Vec<SocketAddr>,
    chain: RwLock<Vec<Header>>,
    blocks: RwLock<Vec<Block>>,
    faults: RwLock<FaultPlan>,
    connection_attempts: AtomicU64,
    protocol_calls: AtomicU64,
    dropped_messages: AtomicU64,
    disconnected_sessions: AtomicU64,
}

impl SimulationNetwork {
    pub fn new(peer_count: usize, block_count: usize, payload_bytes: usize) -> Self {
        let peers = (0..peer_count)
            .map(|offset| {
                SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::LOCALHOST),
                    30_001 + u16::try_from(offset).expect("simulation peer count fits u16"),
                )
            })
            .collect();
        let (chain, blocks) = synthetic_chain(Point::Genesis, 1, block_count, payload_bytes, 1);
        Self {
            inner: Arc::new(Inner {
                peers,
                chain: RwLock::new(chain),
                blocks: RwLock::new(blocks),
                faults: RwLock::new(FaultPlan::default()),
                connection_attempts: AtomicU64::new(0),
                protocol_calls: AtomicU64::new(0),
                dropped_messages: AtomicU64::new(0),
                disconnected_sessions: AtomicU64::new(0),
            }),
        }
    }

    pub fn peers(&self) -> &[SocketAddr] {
        &self.inner.peers
    }

    pub async fn set_faults(&self, faults: FaultPlan) {
        *self.inner.faults.write().await = faults;
    }

    pub async fn tip(&self) -> Tip {
        tip(self.inner.chain.read().await.last())
    }

    pub async fn tip_point(&self) -> Point {
        self.inner
            .chain
            .read()
            .await
            .last()
            .map_or(Point::Genesis, |header| header.point)
    }

    pub async fn headers(&self) -> Vec<Header> {
        self.inner.chain.read().await.clone()
    }

    /// Replace an arbitrary suffix to force connected chain-sync clients to roll back.
    pub async fn replace_suffix(&self, depth: usize, replacement: usize, payload_bytes: usize) {
        let mut chain = self.inner.chain.write().await;
        let mut blocks = self.inner.blocks.write().await;
        let retained = chain.len().saturating_sub(depth);
        chain.truncate(retained);
        blocks.truncate(retained);
        let parent = chain.last().map_or(Point::Genesis, |header| header.point);
        let first_number = chain.last().map_or(1, |header| header.block_number + 1);
        let first_slot = chain.last().map_or(2, |header| header.slot() + 2);
        let (suffix, suffix_blocks) =
            synthetic_chain(parent, first_number, replacement, payload_bytes, first_slot);
        chain.extend(suffix);
        blocks.extend(suffix_blocks);
    }

    pub fn statistics(&self) -> SimulationStatistics {
        SimulationStatistics {
            connection_attempts: self.inner.connection_attempts.load(Ordering::Relaxed),
            protocol_calls: self.inner.protocol_calls.load(Ordering::Relaxed),
            dropped_messages: self.inner.dropped_messages.load(Ordering::Relaxed),
            disconnected_sessions: self.inner.disconnected_sessions.load(Ordering::Relaxed),
        }
    }
}

impl Connector for SimulationNetwork {
    fn connect(&self, peer: SocketAddr) -> BoxFuture<'_, Result<PeerConnection, TransportError>> {
        Box::pin(async move {
            if !self.inner.peers.contains(&peer) {
                return Err(TransportError::Disconnected);
            }
            self.inner
                .connection_attempts
                .fetch_add(1, Ordering::Relaxed);
            let session = Arc::new(Session {
                network: self.clone(),
                calls: AtomicU64::new(0),
                connected: AtomicBool::new(true),
            });
            Ok(PeerConnection::new(
                peer,
                Some(Box::new(SimKeepAlive(session.clone()))),
                Some(Box::new(SimPeerSharing(session.clone()))),
                Box::new(SimChainSync {
                    session: session.clone(),
                    history: Vec::new(),
                    cursor: 0,
                }),
                Box::new(SimBlockFetch(session)),
            ))
        })
    }
}

struct Session {
    network: SimulationNetwork,
    calls: AtomicU64,
    connected: AtomicBool,
}

impl Session {
    async fn call(&self) -> Result<(), TransportError> {
        if !self.connected.load(Ordering::Relaxed) {
            return Err(TransportError::Disconnected);
        }
        let call = self.calls.fetch_add(1, Ordering::Relaxed) + 1;
        self.network
            .inner
            .protocol_calls
            .fetch_add(1, Ordering::Relaxed);
        let faults = self.network.inner.faults.read().await;
        if faults.disconnect_after != 0 && call >= faults.disconnect_after {
            self.connected.store(false, Ordering::Relaxed);
            self.network
                .inner
                .disconnected_sessions
                .fetch_add(1, Ordering::Relaxed);
            return Err(TransportError::Disconnected);
        }
        if faults.drop_calls.contains(&call)
            || (faults.drop_every != 0 && call.is_multiple_of(faults.drop_every))
        {
            self.network
                .inner
                .dropped_messages
                .fetch_add(1, Ordering::Relaxed);
            return Err(TransportError::Dropped);
        }
        Ok(())
    }
}

struct SimKeepAlive(Arc<Session>);

impl KeepAliveClient for SimKeepAlive {
    fn keep_alive(&mut self) -> BoxFuture<'_, Result<(), TransportError>> {
        Box::pin(self.0.call())
    }
}

struct SimPeerSharing(Arc<Session>);

impl PeerSharingClient for SimPeerSharing {
    fn share_peers(
        &mut self,
        amount: u8,
    ) -> BoxFuture<'_, Result<Vec<SocketAddr>, TransportError>> {
        Box::pin(async move {
            self.0.call().await?;
            Ok(self
                .0
                .network
                .inner
                .peers
                .iter()
                .copied()
                .take(amount as usize)
                .collect())
        })
    }
}

struct SimChainSync {
    session: Arc<Session>,
    history: Vec<Point>,
    cursor: usize,
}

impl ChainSyncClient for SimChainSync {
    fn find_intersect(
        &mut self,
        points: &[Point],
    ) -> BoxFuture<'_, Result<Option<Point>, TransportError>> {
        let points = points.to_vec();
        Box::pin(async move {
            self.session.call().await?;
            let chain = self.session.network.inner.chain.read().await;
            let found = points.into_iter().find(|point| {
                *point == Point::Genesis || chain.iter().any(|header| header.point == *point)
            });
            if let Some(point) = found {
                self.cursor = if point == Point::Genesis {
                    0
                } else {
                    chain
                        .iter()
                        .position(|header| header.point == point)
                        .expect("intersection was checked")
                        + 1
                };
                self.history = chain[..self.cursor]
                    .iter()
                    .map(|header| header.point)
                    .collect();
            }
            Ok(found)
        })
    }

    fn next(&mut self) -> BoxFuture<'_, Result<ChainEvent, TransportError>> {
        Box::pin(async move {
            self.session.call().await?;
            loop {
                let chain = self.session.network.inner.chain.read().await;
                let common = self
                    .history
                    .iter()
                    .rposition(|point| chain.iter().any(|header| header.point == *point));
                if common.map_or(!self.history.is_empty(), |index| {
                    index + 1 < self.history.len()
                }) {
                    let point = common.map_or(Point::Genesis, |index| self.history[index]);
                    self.history.truncate(common.map_or(0, |index| index + 1));
                    self.cursor = if point == Point::Genesis {
                        0
                    } else {
                        chain
                            .iter()
                            .position(|header| header.point == point)
                            .expect("common point exists")
                            + 1
                    };
                    return Ok(ChainEvent::RollBackward {
                        point,
                        tip: tip(chain.last()),
                    });
                }
                if let Some(header) = chain.get(self.cursor).cloned() {
                    self.cursor += 1;
                    self.history.push(header.point);
                    return Ok(ChainEvent::RollForward {
                        header,
                        tip: tip(chain.last()),
                    });
                }
                drop(chain);
                tokio::time::sleep(Duration::from_millis(1)).await;
                self.session.call().await?;
            }
        })
    }
}

struct SimBlockFetch(Arc<Session>);

impl BlockFetchClient for SimBlockFetch {
    fn fetch_range(
        &mut self,
        start: Point,
        end: Point,
    ) -> BoxFuture<'_, Result<Vec<Block>, TransportError>> {
        Box::pin(async move {
            self.0.call().await?;
            let blocks = self.0.network.inner.blocks.read().await;
            let start = blocks
                .iter()
                .position(|block| block.header.point == start)
                .ok_or(TransportError::NoBlocks)?;
            let end = blocks
                .iter()
                .position(|block| block.header.point == end)
                .ok_or(TransportError::NoBlocks)?;
            if start > end {
                return Err(TransportError::NoBlocks);
            }
            Ok(blocks[start..=end].to_vec())
        })
    }
}

fn synthetic_chain(
    mut parent: Point,
    first_number: u64,
    count: usize,
    payload_bytes: usize,
    first_slot: u64,
) -> (Vec<Header>, Vec<Block>) {
    let mut headers = Vec::with_capacity(count);
    let mut blocks = Vec::with_capacity(count);
    for offset in 0..count {
        let header = Header::synthetic(
            parent,
            first_number + offset as u64,
            first_slot + offset as u64,
        );
        parent = header.point;
        blocks.push(Block::synthetic(header.clone(), payload_bytes));
        headers.push(header);
    }
    (headers, blocks)
}

fn tip(header: Option<&Header>) -> Tip {
    header.map_or(Tip::Genesis, |header| {
        let Point::Block { slot, hash } = header.point else {
            unreachable!("headers cannot use genesis")
        };
        Tip::Block {
            slot,
            hash,
            block_number: header.block_number,
        }
    })
}
