use network_components::{
    BlockFetcher, BlockFetcherConfig, CardanoConnector, ChainSynchronizer, ChainSynchronizerConfig,
    Connector, PeerManager, PeerManagerConfig, Point, simulation::SimulationNetwork,
};
use std::{
    collections::HashSet,
    error::Error,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if matches!(
        arguments.first().map(String::as_str),
        Some("--preprod" | "--preview")
    ) {
        let network = arguments[0].as_str();
        let address = arguments.get(1).ok_or("missing testnet peer address")?;
        let peer: SocketAddr = match address.parse() {
            Ok(peer) => peer,
            Err(_) => tokio::net::lookup_host(address)
                .await?
                .next()
                .ok_or("preprod peer did not resolve")?,
        };
        let output = arguments
            .get(2)
            .map_or("partial-node.blocks", String::as_str);
        let seconds = arguments
            .get(3)
            .map(|value| value.parse())
            .transpose()?
            .unwrap_or(30 * 60);
        let start_point = match (arguments.get(4), arguments.get(5)) {
            (None, None) => Point::Genesis,
            (Some(slot), Some(hash)) => Point::Block {
                slot: slot.parse()?,
                hash: parse_hash(hash)?,
            },
            _ => return Err("a start point requires both a slot and a 64-character hash".into()),
        };
        let connector = if network == "--preprod" {
            CardanoConnector::preprod()
        } else {
            CardanoConnector::preview()
        };
        run(
            Arc::new(connector),
            vec![peer],
            start_point,
            Point::Genesis,
            Path::new(output),
            Some(Duration::from_secs(seconds)),
        )
        .await
    } else {
        let output = arguments
            .first()
            .map_or("partial-node.blocks", String::as_str);
        let network = SimulationNetwork::new(4, 20_000, 256);
        let tip = network.tip_point().await;
        run(
            Arc::new(network.clone()),
            vec![network.peers()[0]],
            Point::Genesis,
            tip,
            Path::new(output),
            None,
        )
        .await
    }
}

async fn run<C: Connector>(
    connector: Arc<C>,
    bootstrap: Vec<SocketAddr>,
    start_point: Point,
    expected_tip: Point,
    output: &Path,
    run_for: Option<Duration>,
) -> Result<(), Box<dyn Error>> {
    let (peers, _) = PeerManager::spawn(
        PeerManagerConfig {
            bootstrap_peers: bootstrap,
            target_peers: 3,
            maintenance_interval: Duration::from_secs(2),
            peer_share_amount: 8,
            // Keep-Alive is optional in node-to-node and the public relay smoke path focuses on
            // Chain Sync/Block Fetch. The deterministic path exercises it continuously.
            keep_alive: run_for.is_none(),
        },
        connector,
    );
    if let Err(error) = peers.wait_for_active(1, Duration::from_secs(20)).await {
        return Err(format!("{error}; peer state: {:?}", peers.snapshot()).into());
    }
    let (chain, _) = ChainSynchronizer::spawn(
        ChainSynchronizerConfig {
            start_points: if start_point == Point::Genesis {
                vec![Point::Genesis]
            } else {
                vec![start_point, Point::Genesis]
            },
            max_headers: 1_000_000,
            poll_interval: Duration::from_micros(10),
            ..ChainSynchronizerConfig::default()
        },
        peers.clone(),
    );
    let (fetcher, _) = BlockFetcher::spawn(
        BlockFetcherConfig {
            batch_size: 256,
            cache_capacity: 1_000_000,
            ..BlockFetcherConfig::default()
        },
        peers.clone(),
        chain.clone(),
    );

    let started = Instant::now();
    let deadline = run_for.map(|duration| started + duration);
    let mut fetched = HashSet::new();
    loop {
        let headers = chain.headers().await?;
        fetch_missing(&headers, &fetcher, &mut fetched, false).await?;
        let snapshot = chain.snapshot();
        if run_for.is_none()
            && snapshot.selected_tip == expected_tip
            && headers
                .last()
                .is_some_and(|header| header.point == expected_tip)
            && headers.iter().all(|header| fetched.contains(&header.point))
        {
            break;
        }
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let headers = chain.headers().await?;
    fetch_missing(&headers, &fetcher, &mut fetched, true).await?;
    let selected_tip = headers.last().map_or(Point::Genesis, |header| header.point);
    let blocks = write_selected(output, &headers, &fetcher).await?;
    let chain_snapshot = chain.snapshot();
    let fetch_snapshot = fetcher.snapshot();
    let peer_snapshot = peers.snapshot();

    fetcher.shutdown().await?;
    chain.shutdown().await?;
    peers.shutdown().await?;
    verify_output(output, &headers, &blocks).await?;

    println!("selected tip: {selected_tip:?}");
    println!("retained headers: {}", headers.len());
    println!("roll forwards: {}", chain_snapshot.roll_forwards);
    println!("rollbacks: {}", chain_snapshot.rollbacks);
    println!("peer failures: {}", chain_snapshot.peer_failures);
    println!(
        "validation failures: {}",
        chain_snapshot.validation_failures
    );
    println!("chain error: {:?}", chain_snapshot.last_error);
    println!("stored blocks: {}", blocks.len());
    println!("fetch state: {fetch_snapshot:?}");
    println!("peer state: {peer_snapshot:?}");
    println!("elapsed: {:.3?}", started.elapsed());
    println!("output: {}", output.display());
    println!("post-shutdown verification: PASS");
    Ok(())
}

async fn fetch_missing(
    headers: &[network_components::Header],
    fetcher: &network_components::BlockFetcherHandle,
    fetched: &mut HashSet<Point>,
    required: bool,
) -> Result<(), Box<dyn Error>> {
    let missing: Vec<_> = headers
        .iter()
        .filter(|header| !fetched.contains(&header.point))
        .collect();
    for batch in missing.chunks(256) {
        if let (Some(first), Some(last)) = (batch.first(), batch.last()) {
            match fetcher.fetch_range(first.point, last.point).await {
                Ok(_) => fetched.extend(batch.iter().map(|header| header.point)),
                Err(error) if required => return Err(error.into()),
                Err(error) => eprintln!("block fetch failed: {error}"),
            }
        }
    }
    Ok(())
}

async fn write_selected(
    path: &Path,
    headers: &[network_components::Header],
    fetcher: &network_components::BlockFetcherHandle,
) -> Result<Vec<network_components::Block>, Box<dyn Error>> {
    let temporary = temporary_path(path);
    let mut file = tokio::fs::File::create(&temporary).await?;
    let points: Vec<_> = headers.iter().map(|header| header.point).collect();
    let blocks = fetcher.blocks(&points).await?;
    if blocks.len() != headers.len() {
        return Err(format!(
            "only {} of {} selected blocks were cached",
            blocks.len(),
            headers.len()
        )
        .into());
    }
    for (header, block) in headers.iter().zip(&blocks) {
        block.verify_against(header)?;
        file.write_u64(block.cbor.len() as u64).await?;
        file.write_all(&block.cbor).await?;
    }
    file.flush().await?;
    drop(file);
    tokio::fs::rename(&temporary, path).await?;
    Ok(blocks)
}

async fn verify_output(
    path: &Path,
    headers: &[network_components::Header],
    blocks: &[network_components::Block],
) -> Result<(), Box<dyn Error>> {
    let bytes = tokio::fs::read(path).await?;
    let mut offset = 0usize;
    for (header, block) in headers.iter().zip(blocks) {
        block.verify_against(header)?;
        let length_bytes: [u8; 8] = bytes
            .get(offset..offset + 8)
            .ok_or("truncated block length")?
            .try_into()?;
        offset += 8;
        let length = u64::from_be_bytes(length_bytes) as usize;
        let stored = bytes
            .get(offset..offset + length)
            .ok_or("truncated block payload")?;
        if stored != block.cbor {
            return Err("stored block bytes differ from the verified upstream block".into());
        }
        offset += length;
    }
    if offset != bytes.len() {
        return Err("partial-node output has trailing bytes".into());
    }
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut temporary = path.as_os_str().to_os_string();
    temporary.push(".tmp");
    PathBuf::from(temporary)
}

fn parse_hash(value: &str) -> Result<[u8; 32], Box<dyn Error>> {
    if value.len() != 64 || !value.is_ascii() {
        return Err("the start-point hash must contain exactly 64 hexadecimal characters".into());
    }
    let mut hash = [0; 32];
    for (output, pair) in hash.iter_mut().zip(value.as_bytes().chunks_exact(2)) {
        let high = (pair[0] as char)
            .to_digit(16)
            .ok_or("the start-point hash is not hexadecimal")?;
        let low = (pair[1] as char)
            .to_digit(16)
            .ok_or("the start-point hash is not hexadecimal")?;
        *output = ((high << 4) | low) as u8;
    }
    Ok(hash)
}
