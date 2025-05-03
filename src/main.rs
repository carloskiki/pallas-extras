use const_hex::FromHex;
use curve25519_dalek::{EdwardsPoint, Scalar, edwards::CompressedEdwardsY};
use ledger::Block;
use network::{
    NetworkMagic, Point, comatch, hlist_pat, mux,
    protocol::{
        NodeToNode,
        handshake::{
            self,
            message::{NodeToNodeVersionData, VersionTable},
        },
        node_to_node::{block_fetch, chain_sync},
    },
};
use sha2::Digest;
use std::error::Error;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;

pub struct TokioSpawner(pub tokio::runtime::Handle);

impl TokioSpawner {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        TokioSpawner(handle)
    }

    pub fn current() -> Self {
        TokioSpawner::new(tokio::runtime::Handle::current())
    }
}

impl futures::task::Spawn for TokioSpawner {
    fn spawn_obj(
        &self,
        obj: futures::task::FutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        self.0.spawn(obj);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect("preview-node.play.dev.cardano.org:3001")
        .await?
        .compat();
    let hlist_pat![(handshake_client, _), (chain_sync_client, _), (block_fetch_client, _), ...] =
        mux::<NodeToNode>(stream, &TokioSpawner::current())?;

    let handshake_client = handshake_client
        .send(handshake::message::ProposeVersions(
            handshake::message::VersionTable {
                versions: vec![(
                    14,
                    handshake::message::NodeToNodeVersionData {
                        network_magic: NetworkMagic::Preview,
                        diffusion_mode: false,
                        peer_sharing: false,
                        query: false,
                    },
                )],
            },
        ))
        .await?;

    let server_agency = chain_sync_client
        .send(chain_sync::message::FindIntersect {
            points: vec![PREVIEW_ALONZO].into_boxed_slice(),
        })
        .await?;

    let Ok((chain_sync::message::IntersectFound { .. }, chain_sync_client)) =
        server_agency.receive().await?.uninject()
    else {
        return Err("Failed to find intersection".into());
    };

    let Ok((chain_sync::message::RollBackward { .. }, mut chain_sync_client)) = chain_sync_client
        .send(chain_sync::message::Next)
        .await?
        .receive()
        .await?
        .uninject()
    else {
        return Err("Did not roll backward".into());
    };

    let mut last_point = Point::Genesis;
    for _ in 0..10 {
        let server_agency = chain_sync_client.send(chain_sync::message::Next).await?;
        comatch! {server_agency.receive().await?;
            (chain_sync::message::RollForward { header, .. }, new_client) => {
                chain_sync_client = new_client;
                last_point = Point::Block { slot: header.body.slot, hash: todo!() };
            },
            (msg @ chain_sync::message::RollBackward { .. }, _) => {
                dbg!(msg);
                panic!("Need to roll back");
            },
            _ => {
                panic!("Reached AwaitReply");
            }
        }
    }

    let block_fetch_server = block_fetch_client
        .send(block_fetch::message::RequestRange {
            start: PREVIEW_ALONZO,
            end: last_point,
        })
        .await?;
    let mut blocks = Vec::with_capacity(10);
    let Ok((block_fetch::message::StartBatch, mut streaming_server)) = block_fetch_server.receive().await?.uninject() else {
        return Err("Cannot receive block".into());
    };
    
    for _ in 0..10 {
        let Ok((block_fetch::message::Block(block), new_server)) = streaming_server.receive().await?.uninject() else {
            return Err("Cannot receive block".into());
        };
        blocks.push(block);
        
        streaming_server = new_server;
    }

    println!("finished generating test cases.");
    Ok(())
}

const PREVIEW_ALONZO: Point = Point::Genesis;

const PREVIEW_BABBAGE: Point = Point::Block {
    slot: 259180,
    hash: match const_hex::const_decode_to_array(
        b"0ad91d3bbe350b1cfa05b13dba5263c47c5eca4f97b3a3105eba96416785a487",
    ) {
        Ok(hash) => hash,
        Err(_) => unreachable!(),
    },
};
