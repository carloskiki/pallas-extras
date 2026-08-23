use crate::{
    ChainEvent, Header, Point, Tip,
    peer_manager::PeerManagerHandle,
    transport::{PeerConnection, TransportError},
    types::{ValidationError, point_hash},
};
use std::{collections::VecDeque, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast, mpsc, oneshot, watch},
    task::JoinHandle,
    time::{MissedTickBehavior, interval},
};

#[derive(Clone, Debug)]
pub struct ChainSynchronizerConfig {
    /// Candidate intersection points, newest first.
    pub start_points: Vec<Point>,
    /// Maximum retained headers. Zero retains the complete synchronized chain.
    pub max_headers: usize,
    pub poll_interval: Duration,
    pub request_timeout: Duration,
}

impl Default for ChainSynchronizerConfig {
    fn default() -> Self {
        Self {
            start_points: vec![Point::Genesis],
            max_headers: 2_000_000,
            poll_interval: Duration::from_millis(10),
            request_timeout: Duration::from_secs(150),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainSnapshot {
    pub selected_tip: Point,
    pub remote_tip: Tip,
    pub retained_headers: usize,
    pub active_peer: Option<SocketAddr>,
    pub roll_forwards: u64,
    pub rollbacks: u64,
    pub peer_failures: u64,
    pub validation_failures: u64,
    pub last_error: Option<String>,
}

impl Default for ChainSnapshot {
    fn default() -> Self {
        Self {
            selected_tip: Point::Genesis,
            remote_tip: Tip::Genesis,
            retained_headers: 0,
            active_peer: None,
            roll_forwards: 0,
            rollbacks: 0,
            peer_failures: 0,
            validation_failures: 0,
            last_error: None,
        }
    }
}

enum Command {
    Headers(oneshot::Sender<Vec<Header>>),
    Shutdown(oneshot::Sender<()>),
}

#[derive(Clone)]
pub struct ChainSynchronizerHandle {
    commands: mpsc::Sender<Command>,
    snapshot: watch::Receiver<ChainSnapshot>,
    events: broadcast::Sender<ChainEvent>,
}

impl ChainSynchronizerHandle {
    pub fn snapshot(&self) -> ChainSnapshot {
        self.snapshot.borrow().clone()
    }

    pub fn subscribe_state(&self) -> watch::Receiver<ChainSnapshot> {
        self.snapshot.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ChainEvent> {
        self.events.subscribe()
    }

    pub async fn headers(&self) -> Result<Vec<Header>, ChainSynchronizerError> {
        let (sender, receiver) = oneshot::channel();
        self.commands
            .send(Command::Headers(sender))
            .await
            .map_err(|_| ChainSynchronizerError::Stopped)?;
        receiver.await.map_err(|_| ChainSynchronizerError::Stopped)
    }

    pub async fn wait_for_tip(
        &self,
        expected: Point,
        timeout: Duration,
    ) -> Result<ChainSnapshot, ChainSynchronizerError> {
        let mut snapshots = self.snapshot.clone();
        tokio::time::timeout(timeout, async move {
            loop {
                let snapshot = snapshots.borrow().clone();
                if snapshot.selected_tip == expected {
                    return Ok(snapshot);
                }
                snapshots
                    .changed()
                    .await
                    .map_err(|_| ChainSynchronizerError::Stopped)?;
            }
        })
        .await
        .map_err(|_| ChainSynchronizerError::Timeout)?
    }

    pub async fn shutdown(&self) -> Result<(), ChainSynchronizerError> {
        let (sender, receiver) = oneshot::channel();
        self.commands
            .send(Command::Shutdown(sender))
            .await
            .map_err(|_| ChainSynchronizerError::Stopped)?;
        receiver.await.map_err(|_| ChainSynchronizerError::Stopped)
    }
}

pub struct ChainSynchronizer;

impl ChainSynchronizer {
    pub fn spawn(
        config: ChainSynchronizerConfig,
        peers: PeerManagerHandle,
    ) -> (ChainSynchronizerHandle, JoinHandle<()>) {
        let anchor = config
            .start_points
            .last()
            .copied()
            .unwrap_or(Point::Genesis);
        let initial = ChainSnapshot {
            selected_tip: anchor,
            ..ChainSnapshot::default()
        };
        let (commands, receiver) = mpsc::channel(16);
        let (states, snapshot) = watch::channel(initial);
        let (events, _) = broadcast::channel(1024);
        let handle = ChainSynchronizerHandle {
            commands,
            snapshot,
            events: events.clone(),
        };
        let task = tokio::spawn(run(config, peers, receiver, states, events));
        (handle, task)
    }
}

async fn run(
    config: ChainSynchronizerConfig,
    peers: PeerManagerHandle,
    mut commands: mpsc::Receiver<Command>,
    states: watch::Sender<ChainSnapshot>,
    events: broadcast::Sender<ChainEvent>,
) {
    let anchor = config
        .start_points
        .last()
        .copied()
        .unwrap_or(Point::Genesis);
    let mut chain = HeaderChain::new(anchor, config.max_headers);
    let mut snapshot = ChainSnapshot {
        selected_tip: anchor,
        ..ChainSnapshot::default()
    };
    let mut current_connection: Option<Arc<PeerConnection>> = None;
    let mut needs_intersection = true;
    let mut polling = interval(config.poll_interval);
    polling.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let (upstream_sender, mut upstream_receiver) = mpsc::channel(1);
    let mut upstream_task: Option<JoinHandle<()>> = None;

    loop {
        tokio::select! {
            _ = polling.tick() => {
                if upstream_task.is_some() {
                    continue;
                }
                if current_connection.as_ref().is_none_or(|peer| !peer.is_healthy()) {
                    let Ok(connections) = peers.connections().await else {
                        snapshot.peer_failures += 1;
                        states.send_replace(snapshot.clone());
                        continue;
                    };
                    current_connection = connections.into_iter().find(|peer| peer.is_healthy());
                    needs_intersection = true;
                }
                let Some(connection) = current_connection.clone() else {
                    snapshot.active_peer = None;
                    states.send_replace(snapshot.clone());
                    continue;
                };

                if needs_intersection {
                    let points = chain.intersection_points(&config.start_points);
                    let sender = upstream_sender.clone();
                    let timeout = config.request_timeout;
                    upstream_task = Some(tokio::spawn(async move {
                        let result = tokio::time::timeout(timeout, async {
                            connection.chain_sync.lock().await.find_intersect(&points).await
                        })
                        .await
                        .map_err(|_| TransportError::Io("chain-sync intersection timed out".into()))
                        .and_then(|result| result);
                        let _ = sender.send(UpstreamResult::Intersect(connection, result)).await;
                    }));
                } else {
                    let sender = upstream_sender.clone();
                    let timeout = config.request_timeout;
                    upstream_task = Some(tokio::spawn(async move {
                        let result = tokio::time::timeout(timeout, async {
                            connection.chain_sync.lock().await.next().await
                        })
                        .await
                        .map_err(|_| TransportError::Io("chain-sync next timed out".into()))
                        .and_then(|result| result);
                        let _ = sender.send(UpstreamResult::Next(connection, result)).await;
                    }));
                }
            }
            result = upstream_receiver.recv() => {
                upstream_task = None;
                let Some(result) = result else { break };
                match result {
                    UpstreamResult::Intersect(connection, Ok(Some(point))) => {
                        if chain.contains(point) {
                            let _ = chain.rollback(point);
                        } else {
                            chain.reset(point);
                        }
                        needs_intersection = false;
                        snapshot.active_peer = Some(connection.peer);
                        snapshot.selected_tip = chain.tip();
                        snapshot.retained_headers = chain.len();
                    }
                    UpstreamResult::Next(connection, Ok(event)) => {
                        let tip = match &event {
                            ChainEvent::RollForward { tip, .. } | ChainEvent::RollBackward { tip, .. } => *tip,
                        };
                        match chain.apply(&event) {
                            Ok(()) => {
                                snapshot.last_error = None;
                                match &event {
                                    ChainEvent::RollForward { .. } => snapshot.roll_forwards += 1,
                                    ChainEvent::RollBackward { .. } => snapshot.rollbacks += 1,
                                }
                                snapshot.selected_tip = chain.tip();
                                snapshot.remote_tip = tip;
                                snapshot.retained_headers = chain.len();
                                let _ = events.send(event);
                            }
                            Err(error) => {
                                snapshot.last_error = Some(format!(
                                    "upstream header or rollback failed validation: {error}"
                                ));
                                connection.mark_unhealthy();
                                snapshot.validation_failures += 1;
                                current_connection = None;
                                needs_intersection = true;
                            }
                        }
                    }
                    UpstreamResult::Intersect(connection, Ok(None)) => {
                        snapshot.last_error = Some("peer found no requested chain intersection".into());
                        connection.mark_unhealthy();
                        snapshot.peer_failures += 1;
                        current_connection = None;
                        needs_intersection = true;
                    }
                    UpstreamResult::Intersect(connection, Err(error))
                    | UpstreamResult::Next(connection, Err(error)) => {
                        snapshot.last_error = Some(error.to_string());
                        connection.mark_unhealthy();
                        snapshot.peer_failures += 1;
                        current_connection = None;
                        needs_intersection = true;
                    }
                }
                states.send_replace(snapshot.clone());
            }
            command = commands.recv() => {
                let Some(command) = command else { break };
                match command {
                    Command::Headers(sender) => {
                        let _ = sender.send(chain.headers.iter().cloned().collect());
                    }
                    Command::Shutdown(sender) => {
                        if let Some(task) = upstream_task.take() {
                            task.abort();
                        }
                        let _ = sender.send(());
                        break;
                    }
                }
            }
        }
    }
}

enum UpstreamResult {
    Intersect(Arc<PeerConnection>, Result<Option<Point>, TransportError>),
    Next(Arc<PeerConnection>, Result<ChainEvent, TransportError>),
}

#[derive(Clone, Debug)]
pub struct HeaderChain {
    anchor: Point,
    max_headers: usize,
    headers: VecDeque<Header>,
}

impl HeaderChain {
    pub fn new(anchor: Point, max_headers: usize) -> Self {
        Self {
            anchor,
            max_headers,
            headers: VecDeque::new(),
        }
    }

    pub fn apply(&mut self, event: &ChainEvent) -> Result<(), ValidationError> {
        match event {
            ChainEvent::RollForward { header, .. } => self.roll_forward(header.clone()),
            ChainEvent::RollBackward { point, .. } => self.rollback(*point),
        }
    }

    pub fn roll_forward(&mut self, header: Header) -> Result<(), ValidationError> {
        header.verify()?;
        // Origin is an abstract intersection rather than the Byron genesis block hash, so the
        // first historical header legitimately carries a parent hash we cannot derive locally.
        if !(self.tip() == Point::Genesis && self.headers.is_empty())
            && header.parent != point_hash(self.tip())
        {
            return Err(ValidationError::Parent);
        }
        if let Some(previous) = self.headers.back() {
            if header.block_number != previous.block_number + 1 {
                return Err(ValidationError::BlockNumber);
            }
            // Byron epoch-boundary blocks may share a slot with the first regular block in the
            // epoch. Hash linkage and block number still make the ordering unambiguous.
            if header.slot() < previous.slot() {
                return Err(ValidationError::Slot);
            }
        }
        self.headers.push_back(header);
        if self.max_headers != 0
            && self.headers.len() > self.max_headers
            && let Some(pruned) = self.headers.pop_front()
        {
            self.anchor = pruned.point;
        }
        Ok(())
    }

    pub fn rollback(&mut self, point: Point) -> Result<(), ValidationError> {
        if point == self.anchor {
            self.headers.clear();
            return Ok(());
        }
        let Some(position) = self.headers.iter().position(|header| header.point == point) else {
            return Err(ValidationError::UnknownRollback);
        };
        self.headers.truncate(position + 1);
        Ok(())
    }

    pub fn reset(&mut self, anchor: Point) {
        self.anchor = anchor;
        self.headers.clear();
    }

    pub fn contains(&self, point: Point) -> bool {
        point == self.anchor || self.headers.iter().any(|header| header.point == point)
    }

    pub fn tip(&self) -> Point {
        self.headers
            .back()
            .map_or(self.anchor, |header| header.point)
    }

    pub fn len(&self) -> usize {
        self.headers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    pub fn headers(&self) -> impl Iterator<Item = &Header> {
        self.headers.iter()
    }

    fn intersection_points(&self, configured: &[Point]) -> Vec<Point> {
        let mut points: Vec<_> = self
            .headers
            .iter()
            .rev()
            .take(32)
            .map(|header| header.point)
            .collect();
        for point in configured {
            if !points.contains(point) {
                points.push(*point);
            }
        }
        if !points.contains(&self.anchor) {
            points.push(self.anchor);
        }
        if !points.contains(&Point::Genesis) {
            points.push(Point::Genesis);
        }
        points
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ChainSynchronizerError {
    #[error("chain synchronizer actor stopped")]
    Stopped,
    #[error("timed out waiting for the requested chain tip")]
    Timeout,
}
