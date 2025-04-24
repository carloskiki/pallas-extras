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
    let hlist_pat![(handshake_client, _), (chain_sync_client, _), ...] =
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
    let (handshake::message::AcceptVersion(_, data), _) = handshake_client
        .receive()
        .await?
        .uninject()
        .map_err(|_| "version refused")?;
    dbg!("accepted", data);

    let chain_sync_client = chain_sync_client
        .send(chain_sync::message::FindIntersect {
            points: Box::new([Point::Block {
                slot: 399318,
                hash: FromHex::from_hex(
                    "a3c670af07840288101109e6c173781bd516f645016afcabc47a274ac61adf1c",
                )?,
            }]),
        })
        .await?;

    let (chain_sync::message::IntersectFound { .. }, chain_sync_client) = chain_sync_client
        .receive()
        .await?
        .uninject()
        .map_err(|_| "did not find intersect")?;

    let chain_sync_client = chain_sync_client.send(chain_sync::message::Next).await?;

    let (roll_backward @ chain_sync::message::RollBackward { .. }, chain_sync_client) =
        chain_sync_client
            .receive()
            .await?
            .uninject()
            .map_err(|_| "did not roll backward")?;
    dbg!("roll backward: ", roll_backward);

    let chain_sync_client = chain_sync_client.send(chain_sync::message::Next).await?;

    let (chain_sync::message::RollForward { header, .. }, _next) = chain_sync_client
        .receive()
        .await?
        .uninject()
        .map_err(|_| "did not send roll forward")?;
    dbg!("roll forward: ", header);

    Ok(())
}
