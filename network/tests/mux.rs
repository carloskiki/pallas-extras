use duplex::full_duplex;
use futures::{executor::LocalPool, task::SpawnExt};
use network::{
    NetworkMagic, comatch, hlist_pat,
    protocol::{
        NodeToNode,
        handshake::{self, message::NodeToNodeVersionData},
    },
};

mod duplex;

fn main() {
    let mut tp = LocalPool::new();
    let (first, second) = full_duplex();
    // let hlist_pat![(handshake_client, _), _] =
    //     network::mux::mux::<NodeToNode>(first, &tp.spawner()).unwrap();
    // let hlist_pat![(_, handshake_server), _] =
    //     network::mux::mux::<NodeToNode>(second, &tp.spawner()).unwrap();

    // tp.spawner()
    //     .spawn(async move {
    //         let message = handshake::message::ProposeVersions(handshake::message::VersionTable {
    //             versions: vec![(
    //                 14,
    //                 NodeToNodeVersionData {
    //                     network_magic: NetworkMagic::Preview,
    //                     diffusion_mode: false,
    //                     peer_sharing: false,
    //                     query: false,
    //                 },
    //             )],
    //         });
    //         let client = handshake_client.send(message).await.unwrap();
    //         comatch! { client.receive().await.unwrap();
    //             (handshake::message::AcceptVersion(number, version_data), _client) => {
    //                 println!("accepted: {}", number);
    //                 println!("data: {:?}", version_data);
    //             },
    //             _ => panic!("yo"),
    //         }
    //     })
    //     .unwrap();

    // tp.spawner()
    //     .spawn(async move {
    //         let Ok((handshake::message::ProposeVersions(versions), _server)) =
    //             handshake_server.receive().await.unwrap().uninject();
    //     })
    //     .unwrap();

    // tp.run();
}
