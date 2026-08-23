//! Cardano node-to-node transport adapter.
//!
//! This module uses the workspace `network` crate's typed state machines and multiplexer. See
//! [`CardanoConnector`] for the live transport entry point.

use crate::{
    Block, ChainEvent, Header, NODE_TO_NODE_VERSION, Point,
    transport::{
        BlockFetchClient, BoxFuture, ChainSyncClient, Connector, KeepAliveClient, PeerConnection,
        PeerSharingClient, TransportError,
    },
};
use bytes::Bytes;
use ledger::crypto::{blake2::Blake2b256, digest::Digest};
use network::{Agency, Lazy, NetworkMagic, agency::Client, mux::Handle};
use std::{
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU16, Ordering},
    },
};
use tinycbor::{Decode, Encode};

#[derive(Clone, Debug)]
pub struct CardanoConnector {
    pub network_magic: NetworkMagic,
    pub protocol_version: u16,
    /// Advertise initiator-only diffusion mode. This crate currently consumes upstream protocols.
    pub initiator_only: bool,
    pub peer_sharing: bool,
}

impl CardanoConnector {
    pub fn preprod() -> Self {
        Self {
            network_magic: NetworkMagic::Preprod,
            protocol_version: NODE_TO_NODE_VERSION,
            initiator_only: true,
            peer_sharing: false,
        }
    }

    pub fn preview() -> Self {
        Self {
            network_magic: NetworkMagic::Preview,
            protocol_version: NODE_TO_NODE_VERSION,
            initiator_only: true,
            peer_sharing: false,
        }
    }
}

impl Connector for CardanoConnector {
    fn connect(&self, peer: SocketAddr) -> BoxFuture<'_, Result<PeerConnection, TransportError>> {
        Box::pin(async move {
            let stream = tokio::net::TcpStream::connect(peer).await?;
            stream.set_nodelay(true)?;
            let (handles, mut mux_task) =
                network::mux::mux::<network::node_to_node::NodeToNode, _>(stream);
            let (
                (handshake, _),
                (chain_sync, _),
                (block_fetch, _),
                _,
                (keep_alive, _),
                (peer_sharing, _),
            ) = handles;

            let proposal =
                network::handshake::propose::Versions(network::handshake::VersionTable {
                    versions: vec![(
                        self.protocol_version,
                        network::node_to_node::VersionData {
                            network_magic: self.network_magic,
                            diffusion_mode: self.initiator_only,
                            peer_sharing: self.peer_sharing,
                            query: false,
                        },
                    )],
                });
            let confirm = handshake
                .send(&proposal)
                .await
                .ok_or(TransportError::Disconnected)?;
            let confirmation = tokio::select! {
                confirmation = confirm.receive() => match confirmation {
                    Ok(confirmation) => confirmation,
                    Err(error) => {
                        let mux_result = mux_task.await;
                        return Err(TransportError::Malformed(format!(
                            "handshake receive failed ({error}); multiplexer: {mux_result:?}"
                        )));
                    }
                },
                result = &mut mux_task => {
                    return Err(TransportError::Malformed(format!("multiplexer stopped during handshake: {result:?}")));
                }
            };
            match confirmation {
                network::handshake::confirm::Message::Accept(payload, _) => {
                    let accepted = payload.decode().map_err(decode_error)?;
                    if accepted.0 != self.protocol_version {
                        return Err(TransportError::Handshake(format!(
                            "peer selected version {}, expected {}",
                            accepted.0, self.protocol_version
                        )));
                    }
                }
                network::handshake::confirm::Message::Refuse(payload, _) => {
                    return Err(TransportError::Handshake(format!(
                        "{:?}",
                        payload.decode().map_err(decode_error)?
                    )));
                }
                network::handshake::confirm::Message::Reply(_, _) => {
                    return Err(TransportError::Handshake(
                        "peer requested version negotiation retry".into(),
                    ));
                }
            }

            // Activate the established-temperature protocol before requesting a hot protocol.
            // `cardano-node`'s P2P governor expects this ordering for inbound initiator-only
            // connections, and the reference Pallas client also starts Keep-Alive immediately.
            let keep_alive = keep_alive
                .send(&network::node_to_node::keep_alive::KeepAlive { cookie: 0 })
                .await
                .ok_or(TransportError::Disconnected)?;
            let (response, keep_alive) = tokio::select! {
                response = keep_alive.receive() => response.map_err(|error| {
                    TransportError::Malformed(format!("initial keep-alive failed: {error}"))
                })?,
                result = &mut mux_task => {
                    return Err(TransportError::Malformed(format!("multiplexer stopped during initial keep-alive: {result:?}")));
                }
            };
            if response.decode().map_err(decode_error)?.cookie != 0 {
                return Err(TransportError::Malformed(
                    "initial keep-alive cookie mismatch".into(),
                ));
            }

            let mux_status = Arc::new(Mutex::new(None));
            let monitor_status = mux_status.clone();
            tokio::spawn(async move {
                let result = mux_task.await;
                *monitor_status
                    .lock()
                    .expect("mux status lock is not poisoned") = Some(format!("{result:?}"));
            });

            Ok(PeerConnection::new(
                peer,
                Some(Box::new(CardanoKeepAlive {
                    handle: Some(keep_alive),
                    cookie: AtomicU16::new(1),
                    mux_status: mux_status.clone(),
                })),
                self.peer_sharing.then(|| {
                    Box::new(CardanoPeerSharing {
                        handle: Some(peer_sharing),
                        mux_status: mux_status.clone(),
                    }) as Box<dyn PeerSharingClient>
                }),
                Box::new(CardanoChainSync {
                    handle: Some(chain_sync),
                    mux_status: mux_status.clone(),
                }),
                Box::new(CardanoBlockFetch {
                    handle: Some(block_fetch),
                    mux_status,
                }),
            ))
        })
    }
}

struct CardanoKeepAlive {
    handle: Option<Handle<Client, network::node_to_node::keep_alive::Client>>,
    cookie: AtomicU16,
    mux_status: Arc<Mutex<Option<String>>>,
}

impl KeepAliveClient for CardanoKeepAlive {
    fn keep_alive(&mut self) -> BoxFuture<'_, Result<(), TransportError>> {
        Box::pin(async move {
            let cookie = self.cookie.fetch_add(1, Ordering::Relaxed);
            let handle = self.handle.take().ok_or(TransportError::Disconnected)?;
            let response = handle
                .send(&network::node_to_node::keep_alive::KeepAlive { cookie })
                .await
                .ok_or(TransportError::Disconnected)?;
            let (payload, handle) = response
                .receive()
                .await
                .map_err(|error| handle_error(error, &self.mux_status))?;
            let response = payload.decode().map_err(decode_error)?;
            if response.cookie != cookie {
                return Err(TransportError::Malformed(
                    "keep-alive cookie mismatch".into(),
                ));
            }
            self.handle = Some(handle);
            Ok(())
        })
    }
}

struct CardanoPeerSharing {
    handle: Option<Handle<Client, network::node_to_node::peer_sharing::Idle>>,
    mux_status: Arc<Mutex<Option<String>>>,
}

impl PeerSharingClient for CardanoPeerSharing {
    fn share_peers(
        &mut self,
        amount: u8,
    ) -> BoxFuture<'_, Result<Vec<SocketAddr>, TransportError>> {
        Box::pin(async move {
            let handle = self.handle.take().ok_or(TransportError::Disconnected)?;
            let response = handle
                .send(&network::node_to_node::peer_sharing::Request { amount })
                .await
                .ok_or(TransportError::Disconnected)?;
            let (payload, handle) = response
                .receive()
                .await
                .map_err(|error| handle_error(error, &self.mux_status))?;
            let peers = payload.decode().map_err(decode_error)?.peers;
            self.handle = Some(handle);
            Ok(peers)
        })
    }
}

struct CardanoChainSync {
    handle: Option<Handle<Client, network::node_to_node::chain_sync::Idle>>,
    mux_status: Arc<Mutex<Option<String>>>,
}

impl ChainSyncClient for CardanoChainSync {
    fn find_intersect(
        &mut self,
        points: &[Point],
    ) -> BoxFuture<'_, Result<Option<Point>, TransportError>> {
        let points = points.to_vec();
        Box::pin(async move {
            let handle = self.handle.take().ok_or(TransportError::Disconnected)?;
            let response = handle
                .send(&network::node_to_node::chain_sync::idle::FindIntersect { points })
                .await
                .ok_or(TransportError::Disconnected)?;
            match response
                .receive()
                .await
                .map_err(|error| handle_error(error, &self.mux_status))?
            {
                network::node_to_node::chain_sync::intersect::Message::Found(payload, handle) => {
                    let found = payload.decode().map_err(decode_error)?;
                    self.handle = Some(handle);
                    Ok(Some(found.point))
                }
                network::node_to_node::chain_sync::intersect::Message::NotFound(_, handle) => {
                    self.handle = Some(handle);
                    Ok(None)
                }
            }
        })
    }

    fn next(&mut self) -> BoxFuture<'_, Result<ChainEvent, TransportError>> {
        Box::pin(async move {
            let handle = self.handle.take().ok_or(TransportError::Disconnected)?;
            let response = handle
                .send(&network::node_to_node::chain_sync::idle::Next)
                .await
                .ok_or(TransportError::Disconnected)?;
            let message = response
                .receive()
                .await
                .map_err(|error| handle_error(error, &self.mux_status))?;
            match message {
                network::node_to_node::chain_sync::can_await::Message::AwaitReply(_, waiting) => {
                    match waiting
                        .receive()
                        .await
                        .map_err(|error| handle_error(error, &self.mux_status))?
                    {
                        network::node_to_node::chain_sync::reply::Message::RollForward(
                            payload,
                            handle,
                        ) => {
                            let event = roll_forward(&payload)?;
                            self.handle = Some(handle);
                            Ok(event)
                        }
                        network::node_to_node::chain_sync::reply::Message::RollBackward(
                            payload,
                            handle,
                        ) => {
                            let rollback = payload.decode().map_err(decode_error)?;
                            self.handle = Some(handle);
                            Ok(ChainEvent::RollBackward {
                                point: rollback.point,
                                tip: rollback.tip,
                            })
                        }
                    }
                }
                network::node_to_node::chain_sync::can_await::Message::RollForward(
                    payload,
                    handle,
                ) => {
                    let event = roll_forward(&payload)?;
                    self.handle = Some(handle);
                    Ok(event)
                }
                network::node_to_node::chain_sync::can_await::Message::RollBackward(
                    payload,
                    handle,
                ) => {
                    let rollback = payload.decode().map_err(decode_error)?;
                    self.handle = Some(handle);
                    Ok(ChainEvent::RollBackward {
                        point: rollback.point,
                        tip: rollback.tip,
                    })
                }
            }
        })
    }
}

fn roll_forward(
    payload: &Lazy<network::node_to_node::chain_sync::reply::RollForward<'static>>,
) -> Result<ChainEvent, TransportError> {
    let decoded = network::node_to_node::chain_sync::reply::RollForward::decode(
        &mut tinycbor::Decoder(payload.bytes()),
    )
    .map_err(decode_error)?;
    let header = header_from_ledger(&decoded.header)?;
    Ok(ChainEvent::RollForward {
        header,
        tip: decoded.tip,
    })
}

struct CardanoBlockFetch {
    handle: Option<Handle<Client, network::node_to_node::block_fetch::Idle>>,
    mux_status: Arc<Mutex<Option<String>>>,
}

impl BlockFetchClient for CardanoBlockFetch {
    fn fetch_range(
        &mut self,
        start: Point,
        end: Point,
    ) -> BoxFuture<'_, Result<Vec<Block>, TransportError>> {
        Box::pin(async move {
            let handle = self.handle.take().ok_or(TransportError::Disconnected)?;
            let busy = handle
                .send(&network::node_to_node::block_fetch::idle::RequestRange { start, end })
                .await
                .ok_or(TransportError::Disconnected)?;
            let mut streaming = match busy
                .receive()
                .await
                .map_err(|error| handle_error(error, &self.mux_status))?
            {
                network::node_to_node::block_fetch::busy::Message::NoBlocks(_, handle) => {
                    self.handle = Some(handle);
                    return Err(TransportError::NoBlocks);
                }
                network::node_to_node::block_fetch::busy::Message::StartBatch(_, handle) => handle,
            };
            let mut blocks = Vec::new();
            loop {
                match streaming
                    .receive()
                    .await
                    .map_err(|error| handle_error(error, &self.mux_status))?
                {
                    network::node_to_node::block_fetch::streaming::Message::Block(
                        payload,
                        next,
                    ) => {
                        blocks.push(block_from_payload(&payload)?);
                        streaming = next;
                    }
                    network::node_to_node::block_fetch::streaming::Message::BatchDone(_, idle) => {
                        self.handle = Some(idle);
                        return Ok(blocks);
                    }
                }
            }
        })
    }
}

fn block_from_payload(
    payload: &Lazy<network::node_to_node::block_fetch::streaming::Block<'static>>,
) -> Result<Block, TransportError> {
    let decoded = network::node_to_node::block_fetch::streaming::Block::decode(
        &mut tinycbor::Decoder(payload.bytes()),
    )
    .map_err(decode_error)?;
    let header = match &decoded.0 {
        ledger::Block::Boundary(block) => ledger::block::Header::Boundary(block.header.clone()),
        ledger::Block::Byron(block) => ledger::block::Header::Byron((*block.header).clone()),
        ledger::Block::Shelley(block) => ledger::block::Header::Shelley(block.header.clone()),
        ledger::Block::Allegra(block) => ledger::block::Header::Allegra(block.header.clone()),
        ledger::Block::Mary(block) => ledger::block::Header::Mary(block.header.clone()),
        ledger::Block::Alonzo(block) => ledger::block::Header::Alonzo(block.header.clone()),
        ledger::Block::Babbage(block) => ledger::block::Header::Babbage(block.header.clone()),
        ledger::Block::Conway(block) => ledger::block::Header::Conway(block.header.clone()),
    };
    let header = header_from_ledger(&header)?;
    let cbor = tinycbor::to_vec(&decoded.0);
    Ok(Block {
        header,
        cbor: Bytes::from(cbor),
    })
}

fn header_from_ledger(header: &ledger::block::Header<'_>) -> Result<Header, TransportError> {
    let (parent, number, slot, bytes) = match header {
        ledger::block::Header::Boundary(header) => (
            Some(*header.previous_block),
            header.consensus_data.difficulty[0],
            header.consensus_data.epoch.saturating_mul(21_600),
            encode_byron_header(0, header),
        ),
        ledger::block::Header::Byron(header) => (
            Some(*header.previous_block),
            header.consensus_data.difficulty[0],
            header.consensus_data.slot.epoch.saturating_mul(21_600)
                + header.consensus_data.slot.slot,
            encode_byron_header(1, header),
        ),
        ledger::block::Header::Shelley(header) => shelley_fields(&header.body, header)?,
        ledger::block::Header::Allegra(header) => shelley_fields(&header.body, header)?,
        ledger::block::Header::Mary(header) => shelley_fields(&header.body, header)?,
        ledger::block::Header::Alonzo(header) => shelley_fields(&header.body, header)?,
        ledger::block::Header::Babbage(header) => shelley_fields(&header.body, header)?,
        ledger::block::Header::Conway(header) => shelley_fields(&header.body, header)?,
    };
    let hash = hash_encoded(&bytes);
    Header::new(
        Point::Block { slot, hash },
        parent,
        number,
        Bytes::from(bytes),
    )
    .map_err(|error| TransportError::Malformed(error.to_string()))
}

type HeaderFields = (Option<[u8; 32]>, u64, u64, Vec<u8>);

fn shelley_fields<B: ShelleyBody, H: Encode>(
    body: &B,
    header: &H,
) -> Result<HeaderFields, TransportError> {
    Ok((body.parent(), body.number(), body.slot(), encode(header)?))
}

trait ShelleyBody {
    fn number(&self) -> u64;
    fn slot(&self) -> u64;
    fn previous(&self) -> Option<&[u8; 32]>;
    fn parent(&self) -> Option<[u8; 32]> {
        self.previous().copied()
    }
}

macro_rules! shelley_body {
    ($($type:path),+ $(,)?) => {$(
        impl ShelleyBody for $type {
            fn number(&self) -> u64 { self.number }
            fn slot(&self) -> u64 { self.slot }
            fn previous(&self) -> Option<&[u8; 32]> { self.previous }
        }
    )+};
}

shelley_body!(
    ledger::shelley::block::header::Body<'_>,
    ledger::allegra::block::header::Body<'_>,
    ledger::mary::block::header::Body<'_>,
    ledger::alonzo::block::header::Body<'_>,
    ledger::babbage::block::header::Body<'_>,
    ledger::conway::block::header::Body<'_>,
);

fn encode(value: &impl Encode) -> Result<Vec<u8>, TransportError> {
    Ok(tinycbor::to_vec(value))
}

fn encode_byron_header(prefix: u8, header: &impl Encode) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = tinycbor::Encoder(&mut bytes);
    encoder.array(2).expect("vector writes are infallible");
    (prefix as u64)
        .encode(&mut encoder)
        .expect("vector writes are infallible");
    header
        .encode(&mut encoder)
        .expect("vector writes are infallible");
    bytes
}

fn hash_encoded(bytes: &[u8]) -> [u8; 32] {
    Blake2b256::digest(bytes).into()
}

fn handle_error(
    error: network::mux::handle::Error,
    mux_status: &Mutex<Option<String>>,
) -> TransportError {
    let status = mux_status
        .lock()
        .expect("mux status lock is not poisoned")
        .clone()
        .unwrap_or_else(|| "still running".into());
    TransportError::Malformed(format!("{error}; multiplexer: {status}"))
}

fn decode_error(error: impl std::fmt::Debug) -> TransportError {
    TransportError::Malformed(format!("{error:?}"))
}

#[allow(dead_code)]
fn _assert_client_agency<A: Agency>(_handle: Handle<A, impl network::State>) {}
