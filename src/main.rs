use const_hex::FromHex;
use curve25519_dalek::{EdwardsPoint, Scalar, edwards::CompressedEdwardsY};
use network::{
    NetworkMagic, Point, comatch, hlist_pat, mux,
    protocol::{
        NodeToNode,
        handshake::{
            self,
            message::{NodeToNodeVersionData, VersionTable},
        },
        node_to_node::chain_sync,
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
    let hlist_pat![(handshake_client, _), (mut chain_sync_client, _), ...] =
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

    loop {
        let server_agency = chain_sync_client.send(chain_sync::message::Next).await?;
        comatch! {server_agency.receive().await?;
            (chain_sync::message::RollForward { header, .. }, new_client) => {
                chain_sync_client = new_client;
                println!("Received header #{}", header.body.block_number);
            },
            (chain_sync::message::RollBackward { point, .. }, new_client) => {
                chain_sync_client = new_client;
                println!("Roll back to {:?}", point);
            },
            _ => {
                panic!("Reached AwaitReply");
            }
        }
    }
}
