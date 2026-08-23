use network_components::{
    BlockFetcher, BlockFetcherConfig, ChainEvent, ChainSynchronizer, ChainSynchronizerConfig,
    FaultPlan, Header, PeerManager, PeerManagerConfig, Point, Tip, chain_sync::HeaderChain,
    simulation::SimulationNetwork,
};
use std::{collections::BTreeSet, sync::Arc, time::Duration};

fn peer_config(network: &SimulationNetwork, target: usize) -> PeerManagerConfig {
    PeerManagerConfig {
        bootstrap_peers: vec![network.peers()[0]],
        target_peers: target,
        maintenance_interval: Duration::from_millis(2),
        peer_share_amount: 8,
        keep_alive: true,
    }
}

#[tokio::test]
async fn peer_manager_discovers_and_replaces_failed_connections() {
    let network = SimulationNetwork::new(5, 10, 64);
    network
        .set_faults(FaultPlan {
            drop_calls: BTreeSet::from([5]),
            drop_every: 0,
            disconnect_after: 17,
        })
        .await;
    let (manager, task) = PeerManager::spawn(peer_config(&network, 3), Arc::new(network.clone()));
    manager
        .wait_for_active(3, Duration::from_secs(2))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;
    manager
        .wait_for_active(3, Duration::from_secs(2))
        .await
        .unwrap();
    let snapshot = manager.snapshot();
    let statistics = network.statistics();
    assert_eq!(snapshot.active.len(), 3);
    assert!(snapshot.discovered_peers >= 2);
    assert!(snapshot.successful_connections > 3);
    assert!(statistics.dropped_messages > 0);
    assert!(statistics.disconnected_sessions > 0);
    manager.shutdown().await.unwrap();
    task.await.unwrap();
}

#[tokio::test]
async fn chain_sync_reaches_tip_and_survives_large_rollback() {
    let network = SimulationNetwork::new(3, 2_000, 64);
    let (peers, _) = PeerManager::spawn(peer_config(&network, 2), Arc::new(network.clone()));
    peers
        .wait_for_active(2, Duration::from_secs(2))
        .await
        .unwrap();
    let (chain, task) = ChainSynchronizer::spawn(
        ChainSynchronizerConfig {
            max_headers: 0,
            poll_interval: Duration::from_micros(1),
            request_timeout: Duration::from_secs(2),
            ..ChainSynchronizerConfig::default()
        },
        peers.clone(),
    );
    chain
        .wait_for_tip(network.tip_point().await, Duration::from_secs(10))
        .await
        .unwrap();
    network.replace_suffix(1_200, 1_200, 64).await;
    chain
        .wait_for_tip(network.tip_point().await, Duration::from_secs(10))
        .await
        .unwrap();
    let snapshot = chain.snapshot();
    assert_eq!(snapshot.retained_headers, 2_000);
    assert!(snapshot.rollbacks >= 1);
    assert_eq!(snapshot.validation_failures, 0);
    chain.shutdown().await.unwrap();
    task.await.unwrap();
    peers.shutdown().await.unwrap();
}

#[tokio::test]
async fn chain_sync_prefers_a_configured_intersection_to_origin() {
    let network = SimulationNetwork::new(2, 2_000, 64);
    let all_headers = network.headers().await;
    let intersection = all_headers[1_199].point;
    let (peers, _) = PeerManager::spawn(peer_config(&network, 1), Arc::new(network.clone()));
    peers
        .wait_for_active(1, Duration::from_secs(2))
        .await
        .unwrap();
    let (chain, task) = ChainSynchronizer::spawn(
        ChainSynchronizerConfig {
            start_points: vec![intersection, Point::Genesis],
            max_headers: 0,
            poll_interval: Duration::from_micros(1),
            request_timeout: Duration::from_secs(2),
        },
        peers.clone(),
    );
    chain
        .wait_for_tip(network.tip_point().await, Duration::from_secs(5))
        .await
        .unwrap();
    let synchronized = chain.headers().await.unwrap();
    assert_eq!(synchronized.len(), 800);
    assert_eq!(synchronized[0].block_number, 1_201);
    assert_eq!(chain.snapshot().roll_forwards, 800);
    chain.shutdown().await.unwrap();
    task.await.unwrap();
    peers.shutdown().await.unwrap();
}

#[tokio::test]
async fn block_fetch_exceeds_required_rate_and_serves_selected_blocks() {
    let network = SimulationNetwork::new(3, 5_000, 128);
    let (peers, _) = PeerManager::spawn(peer_config(&network, 2), Arc::new(network.clone()));
    peers
        .wait_for_active(2, Duration::from_secs(2))
        .await
        .unwrap();
    let (chain, _) = ChainSynchronizer::spawn(
        ChainSynchronizerConfig {
            max_headers: 0,
            poll_interval: Duration::from_micros(1),
            ..ChainSynchronizerConfig::default()
        },
        peers.clone(),
    );
    chain
        .wait_for_tip(network.tip_point().await, Duration::from_secs(15))
        .await
        .unwrap();
    let headers = chain.headers().await.unwrap();
    let (fetcher, _) = BlockFetcher::spawn(
        BlockFetcherConfig {
            batch_size: 257,
            cache_capacity: headers.len(),
            request_timeout: Duration::from_secs(2),
        },
        peers.clone(),
        chain.clone(),
    );
    let receipt = fetcher
        .fetch_range(headers[0].point, headers.last().unwrap().point)
        .await
        .unwrap();
    assert_eq!(receipt.blocks, headers.len());
    assert!(receipt.blocks_per_minute() >= 20_000.0);
    let middle = &headers[headers.len() / 2];
    let block = fetcher.block(middle.point).await.unwrap().unwrap();
    block.verify_against(middle).unwrap();
    assert!(fetcher.block(Point::Genesis).await.unwrap().is_none());
    fetcher.shutdown().await.unwrap();
    chain.shutdown().await.unwrap();
    peers.shutdown().await.unwrap();
}

#[test]
fn header_chain_rejects_invalid_links_and_rolls_back_to_any_retained_point() {
    let first = Header::synthetic(Point::Genesis, 1, 1);
    let second = Header::synthetic(first.point, 2, 2);
    let third = Header::synthetic(second.point, 3, 3);
    let hash = match third.point {
        Point::Block { hash, .. } => hash,
        Point::Genesis => unreachable!(),
    };
    let tip = Tip::Block {
        slot: 3,
        hash,
        block_number: 3,
    };
    let mut chain = HeaderChain::new(Point::Genesis, 0);
    for header in [&first, &second, &third] {
        chain
            .apply(&ChainEvent::RollForward {
                header: header.clone(),
                tip,
            })
            .unwrap();
    }
    chain
        .apply(&ChainEvent::RollBackward {
            point: first.point,
            tip,
        })
        .unwrap();
    assert_eq!(chain.tip(), first.point);
    let invalid = Header::synthetic(Point::Genesis, 4, 4);
    assert!(chain.roll_forward(invalid).is_err());
    assert!(chain.rollback(third.point).is_err());
}
