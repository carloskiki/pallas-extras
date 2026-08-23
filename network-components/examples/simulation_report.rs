use network_components::{
    BlockFetcher, BlockFetcherConfig, ChainSynchronizer, ChainSynchronizerConfig, FaultPlan,
    PeerManager, PeerManagerConfig, simulation::SimulationNetwork,
};
use std::{
    collections::BTreeSet,
    error::Error,
    fmt::Write as _,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("reviewing/M3/simulation-report.txt"));

    let resilience = SimulationNetwork::new(6, 8, 64);
    resilience
        .set_faults(FaultPlan {
            drop_calls: BTreeSet::from([3, 11]),
            drop_every: 13,
            disconnect_after: 23,
        })
        .await;
    let (resilient_peers, _) = PeerManager::spawn(
        PeerManagerConfig {
            bootstrap_peers: vec![resilience.peers()[0]],
            target_peers: 4,
            maintenance_interval: Duration::from_millis(2),
            peer_share_amount: 6,
            keep_alive: true,
        },
        Arc::new(resilience.clone()),
    );
    resilient_peers
        .wait_for_active(4, Duration::from_secs(5))
        .await?;
    tokio::time::sleep(Duration::from_millis(400)).await;
    let resilience_snapshot = resilient_peers.snapshot();
    let resilience_stats = resilience.statistics();
    resilient_peers.shutdown().await?;

    const BLOCKS: usize = 30_000;
    const ROLLBACK: usize = 10_000;
    let network = SimulationNetwork::new(4, BLOCKS, 256);
    let (peers, _) = PeerManager::spawn(
        PeerManagerConfig {
            bootstrap_peers: vec![network.peers()[0]],
            target_peers: 3,
            maintenance_interval: Duration::from_millis(2),
            peer_share_amount: 4,
            keep_alive: true,
        },
        Arc::new(network.clone()),
    );
    peers.wait_for_active(3, Duration::from_secs(5)).await?;
    let (chain, _) = ChainSynchronizer::spawn(
        ChainSynchronizerConfig {
            max_headers: 0,
            poll_interval: Duration::from_micros(1),
            request_timeout: Duration::from_secs(10),
            ..ChainSynchronizerConfig::default()
        },
        peers.clone(),
    );
    let sync_started = Instant::now();
    chain
        .wait_for_tip(network.tip_point().await, Duration::from_secs(60))
        .await?;
    let initial_sync = sync_started.elapsed();

    network.replace_suffix(ROLLBACK, ROLLBACK, 256).await;
    let rollback_started = Instant::now();
    chain
        .wait_for_tip(network.tip_point().await, Duration::from_secs(60))
        .await?;
    let rollback_recovery = rollback_started.elapsed();
    let chain_snapshot = chain.snapshot();
    let headers = chain.headers().await?;

    let (fetcher, _) = BlockFetcher::spawn(
        BlockFetcherConfig {
            batch_size: 512,
            cache_capacity: BLOCKS,
            request_timeout: Duration::from_secs(10),
        },
        peers.clone(),
        chain.clone(),
    );
    let receipt = fetcher
        .fetch_range(
            headers.first().expect("chain is non-empty").point,
            headers.last().expect("chain is non-empty").point,
        )
        .await?;
    let first = fetcher
        .block(headers[0].point)
        .await?
        .expect("first block is cached");
    first.verify_against(&headers[0])?;

    let resilience_pass = resilience_snapshot.active.len() == 4
        && resilience_stats.dropped_messages > 0
        && resilience_stats.disconnected_sessions > 0
        && resilience_snapshot.successful_connections > 4;
    let chain_pass = chain_snapshot.selected_tip == network.tip_point().await
        && chain_snapshot.rollbacks >= 1
        && chain_snapshot.validation_failures == 0;
    let fetch_pass = receipt.blocks == BLOCKS && receipt.blocks_per_minute() >= 20_000.0;
    let overall = resilience_pass && chain_pass && fetch_pass;

    let mut report = String::new();
    writeln!(report, "Pallas Extras M3 deterministic simulation report")?;
    writeln!(report, "format: 1")?;
    writeln!(
        report,
        "node_to_node_version: {}",
        network_components::NODE_TO_NODE_VERSION
    )?;
    writeln!(
        report,
        "cardano_node_reference: {}",
        network_components::CARDANO_NODE_VERSION
    )?;
    writeln!(report)?;
    writeln!(report, "[peer_manager]")?;
    writeln!(report, "target_peers: 4")?;
    writeln!(
        report,
        "active_at_completion: {}",
        resilience_snapshot.active.len()
    )?;
    writeln!(
        report,
        "connection_attempts: {}",
        resilience_stats.connection_attempts
    )?;
    writeln!(
        report,
        "successful_connections: {}",
        resilience_snapshot.successful_connections
    )?;
    writeln!(
        report,
        "dropped_messages: {}",
        resilience_stats.dropped_messages
    )?;
    writeln!(
        report,
        "disconnected_sessions: {}",
        resilience_stats.disconnected_sessions
    )?;
    writeln!(report, "result: {}", pass(resilience_pass))?;
    writeln!(report)?;
    writeln!(report, "[chain_synchronizer]")?;
    writeln!(report, "headers: {BLOCKS}")?;
    writeln!(
        report,
        "initial_sync_milliseconds: {}",
        initial_sync.as_millis()
    )?;
    writeln!(report, "rollback_depth: {ROLLBACK}")?;
    writeln!(
        report,
        "rollback_recovery_milliseconds: {}",
        rollback_recovery.as_millis()
    )?;
    writeln!(report, "observed_rollbacks: {}", chain_snapshot.rollbacks)?;
    writeln!(
        report,
        "validation_failures: {}",
        chain_snapshot.validation_failures
    )?;
    writeln!(report, "result: {}", pass(chain_pass))?;
    writeln!(report)?;
    writeln!(report, "[block_fetcher]")?;
    writeln!(report, "blocks: {}", receipt.blocks)?;
    writeln!(report, "bytes: {}", receipt.bytes)?;
    writeln!(
        report,
        "elapsed_milliseconds: {}",
        receipt.elapsed.as_millis()
    )?;
    writeln!(
        report,
        "blocks_per_minute: {:.0}",
        receipt.blocks_per_minute()
    )?;
    writeln!(report, "required_blocks_per_minute: 20000")?;
    writeln!(report, "result: {}", pass(fetch_pass))?;
    writeln!(report)?;
    writeln!(report, "overall: {}", pass(overall))?;

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, &report)?;
    print!("{report}");

    fetcher.shutdown().await?;
    chain.shutdown().await?;
    peers.shutdown().await?;
    if !overall {
        return Err("simulation acceptance check failed".into());
    }
    Ok(())
}

fn pass(value: bool) -> &'static str {
    if value { "PASS" } else { "FAIL" }
}
