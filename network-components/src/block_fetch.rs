use crate::{
    Block, Header, Point, chain_sync::ChainSynchronizerHandle, peer_manager::PeerManagerHandle,
    types::ValidationError,
};
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
};

#[derive(Clone, Debug)]
pub struct BlockFetcherConfig {
    pub batch_size: usize,
    pub cache_capacity: usize,
    pub request_timeout: Duration,
}

impl Default for BlockFetcherConfig {
    fn default() -> Self {
        Self {
            batch_size: 512,
            cache_capacity: 4096,
            request_timeout: Duration::from_secs(60),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockFetcherSnapshot {
    pub cached_blocks: usize,
    pub fetched_blocks: u64,
    pub fetched_bytes: u64,
    pub served_blocks: u64,
    pub failed_batches: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FetchReceipt {
    pub blocks: usize,
    pub bytes: u64,
    pub elapsed: Duration,
}

impl FetchReceipt {
    pub fn blocks_per_minute(&self) -> f64 {
        if self.elapsed.is_zero() {
            return f64::INFINITY;
        }
        self.blocks as f64 * 60.0 / self.elapsed.as_secs_f64()
    }
}

enum Command {
    Fetch {
        start: Point,
        end: Point,
        response: oneshot::Sender<Result<FetchReceipt, BlockFetcherError>>,
    },
    Block {
        point: Point,
        response: oneshot::Sender<Result<Option<Block>, BlockFetcherError>>,
    },
    Blocks {
        points: Vec<Point>,
        response: oneshot::Sender<Result<Vec<Block>, BlockFetcherError>>,
    },
    Shutdown(oneshot::Sender<()>),
}

#[derive(Clone)]
pub struct BlockFetcherHandle {
    commands: mpsc::Sender<Command>,
    snapshot: watch::Receiver<BlockFetcherSnapshot>,
}

impl BlockFetcherHandle {
    pub async fn fetch_range(
        &self,
        start: Point,
        end: Point,
    ) -> Result<FetchReceipt, BlockFetcherError> {
        let (sender, receiver) = oneshot::channel();
        self.commands
            .send(Command::Fetch {
                start,
                end,
                response: sender,
            })
            .await
            .map_err(|_| BlockFetcherError::Stopped)?;
        receiver.await.map_err(|_| BlockFetcherError::Stopped)?
    }

    /// Serve a cached block only if its header remains on the selected chain.
    pub async fn block(&self, point: Point) -> Result<Option<Block>, BlockFetcherError> {
        let (sender, receiver) = oneshot::channel();
        self.commands
            .send(Command::Block {
                point,
                response: sender,
            })
            .await
            .map_err(|_| BlockFetcherError::Stopped)?;
        receiver.await.map_err(|_| BlockFetcherError::Stopped)?
    }

    /// Serve multiple cached, selected blocks with one chain-state snapshot.
    pub async fn blocks(&self, points: &[Point]) -> Result<Vec<Block>, BlockFetcherError> {
        let (sender, receiver) = oneshot::channel();
        self.commands
            .send(Command::Blocks {
                points: points.to_vec(),
                response: sender,
            })
            .await
            .map_err(|_| BlockFetcherError::Stopped)?;
        receiver.await.map_err(|_| BlockFetcherError::Stopped)?
    }

    pub fn snapshot(&self) -> BlockFetcherSnapshot {
        self.snapshot.borrow().clone()
    }

    pub fn subscribe(&self) -> watch::Receiver<BlockFetcherSnapshot> {
        self.snapshot.clone()
    }

    pub async fn shutdown(&self) -> Result<(), BlockFetcherError> {
        let (sender, receiver) = oneshot::channel();
        self.commands
            .send(Command::Shutdown(sender))
            .await
            .map_err(|_| BlockFetcherError::Stopped)?;
        receiver.await.map_err(|_| BlockFetcherError::Stopped)
    }
}

pub struct BlockFetcher;

impl BlockFetcher {
    pub fn spawn(
        config: BlockFetcherConfig,
        peers: PeerManagerHandle,
        chain: ChainSynchronizerHandle,
    ) -> (BlockFetcherHandle, JoinHandle<()>) {
        let (commands, receiver) = mpsc::channel(32);
        let (states, snapshot) = watch::channel(BlockFetcherSnapshot::default());
        let handle = BlockFetcherHandle { commands, snapshot };
        let task = tokio::spawn(run(config, peers, chain, receiver, states));
        (handle, task)
    }
}

async fn run(
    config: BlockFetcherConfig,
    peers: PeerManagerHandle,
    chain: ChainSynchronizerHandle,
    mut commands: mpsc::Receiver<Command>,
    states: watch::Sender<BlockFetcherSnapshot>,
) {
    let mut cache = HashMap::<Point, Block>::new();
    let mut order = VecDeque::<Point>::new();
    let mut snapshot = BlockFetcherSnapshot::default();

    while let Some(command) = commands.recv().await {
        match command {
            Command::Fetch {
                start,
                end,
                response,
            } => {
                let result = fetch(
                    &config,
                    &peers,
                    &chain,
                    &mut cache,
                    &mut order,
                    &mut snapshot,
                    start,
                    end,
                )
                .await;
                states.send_replace(snapshot.clone());
                let _ = response.send(result);
            }
            Command::Block { point, response } => {
                let selected = chain
                    .headers()
                    .await
                    .map(|headers| headers.iter().any(|header| header.point == point))
                    .map_err(|_| BlockFetcherError::Stopped);
                let result = selected.map(|selected| {
                    if selected {
                        let block = cache.get(&point).cloned();
                        if block.is_some() {
                            snapshot.served_blocks += 1;
                        }
                        block
                    } else {
                        None
                    }
                });
                states.send_replace(snapshot.clone());
                let _ = response.send(result);
            }
            Command::Blocks { points, response } => {
                let result = chain
                    .headers()
                    .await
                    .map_err(|_| BlockFetcherError::Stopped)
                    .map(|headers| {
                        let selected: std::collections::HashSet<_> =
                            headers.iter().map(|header| header.point).collect();
                        let blocks: Vec<_> = points
                            .into_iter()
                            .filter(|point| selected.contains(point))
                            .filter_map(|point| cache.get(&point).cloned())
                            .collect();
                        snapshot.served_blocks += blocks.len() as u64;
                        blocks
                    });
                states.send_replace(snapshot.clone());
                let _ = response.send(result);
            }
            Command::Shutdown(sender) => {
                let _ = sender.send(());
                break;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn fetch(
    config: &BlockFetcherConfig,
    peers: &PeerManagerHandle,
    chain: &ChainSynchronizerHandle,
    cache: &mut HashMap<Point, Block>,
    order: &mut VecDeque<Point>,
    snapshot: &mut BlockFetcherSnapshot,
    start: Point,
    end: Point,
) -> Result<FetchReceipt, BlockFetcherError> {
    if config.batch_size == 0 {
        return Err(BlockFetcherError::InvalidBatchSize);
    }
    let headers = chain
        .headers()
        .await
        .map_err(|_| BlockFetcherError::Stopped)?;
    let expected = selected_range(&headers, start, end)?;
    let started = Instant::now();
    let mut fetched_blocks = 0usize;
    let mut fetched_bytes = 0u64;

    for batch in expected.chunks(config.batch_size) {
        let connections = peers
            .connections()
            .await
            .map_err(|_| BlockFetcherError::NoPeers)?;
        let Some(connection) = connections.into_iter().find(|peer| peer.is_healthy()) else {
            return Err(BlockFetcherError::NoPeers);
        };
        let first = batch.first().expect("chunks are non-empty");
        let last = batch.last().expect("chunks are non-empty");
        let result = tokio::time::timeout(
            config.request_timeout,
            connection
                .block_fetch
                .lock()
                .await
                .fetch_range(first.point, last.point),
        )
        .await;
        let blocks = match result {
            Ok(Ok(blocks)) => blocks,
            _ => {
                connection.mark_unhealthy();
                snapshot.failed_batches += 1;
                return Err(BlockFetcherError::Transport);
            }
        };
        if blocks.len() != batch.len() {
            connection.mark_unhealthy();
            snapshot.failed_batches += 1;
            return Err(BlockFetcherError::IncompleteBatch {
                expected: batch.len(),
                actual: blocks.len(),
            });
        }
        for (block, header) in blocks.into_iter().zip(batch) {
            block.verify_against(header)?;
            fetched_blocks += 1;
            fetched_bytes += block.cbor.len() as u64;
            insert_cache(config.cache_capacity, cache, order, block);
        }
    }

    snapshot.cached_blocks = cache.len();
    snapshot.fetched_blocks += fetched_blocks as u64;
    snapshot.fetched_bytes += fetched_bytes;
    Ok(FetchReceipt {
        blocks: fetched_blocks,
        bytes: fetched_bytes,
        elapsed: started.elapsed(),
    })
}

fn selected_range(
    headers: &[Header],
    start: Point,
    end: Point,
) -> Result<&[Header], BlockFetcherError> {
    let start = headers
        .iter()
        .position(|header| header.point == start)
        .ok_or(BlockFetcherError::PointNotSelected(start))?;
    let end = headers
        .iter()
        .position(|header| header.point == end)
        .ok_or(BlockFetcherError::PointNotSelected(end))?;
    if start > end {
        return Err(BlockFetcherError::ReverseRange);
    }
    Ok(&headers[start..=end])
}

fn insert_cache(
    capacity: usize,
    cache: &mut HashMap<Point, Block>,
    order: &mut VecDeque<Point>,
    block: Block,
) {
    if capacity == 0 {
        return;
    }
    let point = block.header.point;
    if cache.insert(point, block).is_none() {
        order.push_back(point);
    }
    while cache.len() > capacity {
        if let Some(point) = order.pop_front() {
            cache.remove(&point);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BlockFetcherError {
    #[error("block fetcher actor stopped")]
    Stopped,
    #[error("no healthy upstream peers are available")]
    NoPeers,
    #[error("batch size must be greater than zero")]
    InvalidBatchSize,
    #[error("point is not on the selected chain: {0:?}")]
    PointNotSelected(Point),
    #[error("block range end precedes its start")]
    ReverseRange,
    #[error("upstream returned {actual} blocks for a batch of {expected}")]
    IncompleteBatch { expected: usize, actual: usize },
    #[error("block-fetch mini-protocol failed")]
    Transport,
    #[error(transparent)]
    Validation(#[from] ValidationError),
}
