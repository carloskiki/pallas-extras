use futures::executor::LocalPool;
use network::{
    NetworkMagic, comatch, hlist_pat, mux,
    protocol::{
        NodeToNode,
        handshake::{
            self,
            message::{NodeToNodeVersionData, VersionTable},
        },
    },
};
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
    let hlist_pat![(handshake_client, _), _, _] =
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

    Ok(())
}
