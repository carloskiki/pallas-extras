use crate::transport::{Connector, PeerConnection, TransportError};
use std::{
    collections::{BTreeSet, HashMap},
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
    time::{MissedTickBehavior, interval},
};

#[derive(Clone, Debug)]
pub struct PeerManagerConfig {
    pub bootstrap_peers: Vec<SocketAddr>,
    pub target_peers: usize,
    pub maintenance_interval: Duration,
    pub peer_share_amount: u8,
    pub keep_alive: bool,
}

impl Default for PeerManagerConfig {
    fn default() -> Self {
        Self {
            bootstrap_peers: Vec::new(),
            target_peers: 3,
            maintenance_interval: Duration::from_secs(10),
            peer_share_amount: 8,
            keep_alive: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PeerSnapshot {
    pub active: Vec<SocketAddr>,
    pub available: Vec<SocketAddr>,
    pub connection_attempts: u64,
    pub successful_connections: u64,
    pub lost_connections: u64,
    pub discovered_peers: u64,
    pub dropped_messages: u64,
    pub last_error: Option<String>,
}

enum Command {
    Add(SocketAddr),
    Disconnect(SocketAddr),
    Connections(oneshot::Sender<Vec<Arc<PeerConnection>>>),
    Shutdown(oneshot::Sender<()>),
}

#[derive(Clone)]
pub struct PeerManagerHandle {
    commands: mpsc::Sender<Command>,
    snapshot: watch::Receiver<PeerSnapshot>,
}

impl PeerManagerHandle {
    pub async fn add_peer(&self, peer: SocketAddr) -> Result<(), PeerManagerError> {
        self.commands
            .send(Command::Add(peer))
            .await
            .map_err(|_| PeerManagerError::Stopped)
    }

    pub async fn disconnect(&self, peer: SocketAddr) -> Result<(), PeerManagerError> {
        self.commands
            .send(Command::Disconnect(peer))
            .await
            .map_err(|_| PeerManagerError::Stopped)
    }

    pub async fn connections(&self) -> Result<Vec<Arc<PeerConnection>>, PeerManagerError> {
        let (sender, receiver) = oneshot::channel();
        self.commands
            .send(Command::Connections(sender))
            .await
            .map_err(|_| PeerManagerError::Stopped)?;
        receiver.await.map_err(|_| PeerManagerError::Stopped)
    }

    pub fn snapshot(&self) -> PeerSnapshot {
        self.snapshot.borrow().clone()
    }

    pub fn subscribe(&self) -> watch::Receiver<PeerSnapshot> {
        self.snapshot.clone()
    }

    pub async fn wait_for_active(
        &self,
        minimum: usize,
        timeout: Duration,
    ) -> Result<PeerSnapshot, PeerManagerError> {
        let mut snapshots = self.snapshot.clone();
        tokio::time::timeout(timeout, async move {
            loop {
                let snapshot = snapshots.borrow().clone();
                if snapshot.active.len() >= minimum {
                    return Ok(snapshot);
                }
                snapshots
                    .changed()
                    .await
                    .map_err(|_| PeerManagerError::Stopped)?;
            }
        })
        .await
        .map_err(|_| PeerManagerError::Timeout)?
    }

    pub async fn shutdown(&self) -> Result<(), PeerManagerError> {
        let (sender, receiver) = oneshot::channel();
        self.commands
            .send(Command::Shutdown(sender))
            .await
            .map_err(|_| PeerManagerError::Stopped)?;
        receiver.await.map_err(|_| PeerManagerError::Stopped)
    }
}

pub struct PeerManager;

impl PeerManager {
    pub fn spawn<C: Connector>(
        config: PeerManagerConfig,
        connector: Arc<C>,
    ) -> (PeerManagerHandle, JoinHandle<()>) {
        let (commands, receiver) = mpsc::channel(64);
        let initial = PeerSnapshot {
            available: config.bootstrap_peers.clone(),
            ..PeerSnapshot::default()
        };
        let (snapshots, snapshot) = watch::channel(initial);
        let handle = PeerManagerHandle { commands, snapshot };
        let task = tokio::spawn(run(config, connector, receiver, snapshots));
        (handle, task)
    }
}

async fn run<C: Connector>(
    config: PeerManagerConfig,
    connector: Arc<C>,
    mut commands: mpsc::Receiver<Command>,
    snapshots: watch::Sender<PeerSnapshot>,
) {
    let mut available: BTreeSet<_> = config.bootstrap_peers.iter().copied().collect();
    let mut active = HashMap::<SocketAddr, Arc<PeerConnection>>::new();
    let mut statistics = PeerSnapshot::default();
    let mut maintenance = interval(config.maintenance_interval);
    maintenance.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = maintenance.tick() => {
                maintain(
                    &config,
                    connector.as_ref(),
                    &mut available,
                    &mut active,
                    &mut statistics,
                ).await;
                publish(&snapshots, &available, &active, &statistics);
            }
            command = commands.recv() => {
                let Some(command) = command else { break };
                match command {
                    Command::Add(peer) => {
                        if !active.contains_key(&peer) {
                            available.insert(peer);
                        }
                    }
                    Command::Disconnect(peer) => {
                        if let Some(connection) = active.remove(&peer) {
                            connection.mark_unhealthy();
                            statistics.lost_connections += 1;
                            available.insert(peer);
                        }
                    }
                    Command::Connections(sender) => {
                        let _ = sender.send(active.values().filter(|peer| peer.is_healthy()).cloned().collect());
                    }
                    Command::Shutdown(sender) => {
                        for connection in active.values() {
                            connection.mark_unhealthy();
                        }
                        let _ = sender.send(());
                        break;
                    }
                }
                publish(&snapshots, &available, &active, &statistics);
            }
        }
    }
}

async fn maintain<C: Connector>(
    config: &PeerManagerConfig,
    connector: &C,
    available: &mut BTreeSet<SocketAddr>,
    active: &mut HashMap<SocketAddr, Arc<PeerConnection>>,
    statistics: &mut PeerSnapshot,
) {
    let peers: Vec<_> = active.values().cloned().collect();
    for connection in peers {
        let mut disconnected = !connection.is_healthy();

        if !disconnected
            && config.keep_alive
            && let Some(keep_alive) = &connection.keep_alive
        {
            match keep_alive.lock().await.keep_alive().await {
                Ok(()) => {}
                Err(TransportError::Dropped) => statistics.dropped_messages += 1,
                Err(error) => {
                    statistics.last_error = Some(error.to_string());
                    disconnected = true;
                }
            }
        }

        if !disconnected && let Some(peer_sharing) = &connection.peer_sharing {
            match peer_sharing
                .lock()
                .await
                .share_peers(config.peer_share_amount)
                .await
            {
                Ok(discovered) => {
                    for peer in discovered {
                        if !active.contains_key(&peer) && available.insert(peer) {
                            statistics.discovered_peers += 1;
                        }
                    }
                }
                Err(TransportError::Dropped) => statistics.dropped_messages += 1,
                Err(error) => {
                    statistics.last_error = Some(error.to_string());
                    disconnected = true;
                }
            }
        }

        if disconnected {
            connection.mark_unhealthy();
            active.remove(&connection.peer);
            available.insert(connection.peer);
            statistics.lost_connections += 1;
        }
    }

    while active.len() < config.target_peers {
        let Some(peer) = available.pop_first() else {
            break;
        };
        statistics.connection_attempts += 1;
        match connector.connect(peer).await {
            Ok(connection) => {
                statistics.successful_connections += 1;
                statistics.last_error = None;
                active.insert(peer, Arc::new(connection));
            }
            Err(TransportError::Dropped) => {
                statistics.dropped_messages += 1;
                available.insert(peer);
                break;
            }
            Err(error) => {
                statistics.last_error = Some(error.to_string());
                available.insert(peer);
                break;
            }
        }
    }
}

fn publish(
    sender: &watch::Sender<PeerSnapshot>,
    available: &BTreeSet<SocketAddr>,
    active: &HashMap<SocketAddr, Arc<PeerConnection>>,
    statistics: &PeerSnapshot,
) {
    let mut snapshot = statistics.clone();
    snapshot.active = active
        .values()
        .filter(|peer| peer.is_healthy())
        .map(|peer| peer.peer)
        .collect();
    snapshot.active.sort_unstable();
    snapshot.available = available.iter().copied().collect();
    sender.send_replace(snapshot);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PeerManagerError {
    #[error("peer manager actor stopped")]
    Stopped,
    #[error("timed out waiting for active peers")]
    Timeout,
}
