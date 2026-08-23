use network::{
    NetworkMagic,
    handshake::{VersionTable, confirm::Accept, propose::Versions},
    node_to_node::{self, NodeToNode, VersionData},
};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

fn version_data() -> VersionData {
    VersionData {
        network_magic: NetworkMagic::Preview,
        diffusion_mode: false,
        peer_sharing: true,
        query: false,
    }
}

#[tokio::test(flavor = "current_thread")]
async fn handshake_crosses_a_fragmented_duplex_bearer() {
    let (client_bearer, server_bearer) = tokio::io::duplex(11);
    let (client_handles, client_mux) = network::mux::mux::<NodeToNode, _>(client_bearer);
    let (server_handles, server_mux) = network::mux::mux::<NodeToNode, _>(server_bearer);
    let ((client, _), ..) = client_handles;
    let ((_, server), ..) = server_handles;

    let client = tokio::spawn(async move {
        let confirm = client
            .send(&Versions(VersionTable {
                versions: vec![(14, version_data())],
            }))
            .await
            .unwrap();
        match confirm.receive().await.unwrap() {
            network::handshake::confirm::Message::Accept(payload, _) => {
                let accept = payload.decode().unwrap();
                assert_eq!(accept.0, 14);
            }
            network::handshake::confirm::Message::Refuse(_, _) => panic!("handshake refused"),
            network::handshake::confirm::Message::Reply(_, _) => panic!("unexpected retry"),
        }
    });
    let server = tokio::spawn(async move {
        let (payload, confirm) = server.receive().await.unwrap();
        let proposal = payload.decode().unwrap();
        assert_eq!(proposal.0.versions.len(), 1);
        assert_eq!(proposal.0.versions[0].0, 14);
        confirm.send(&Accept(14, version_data())).await.unwrap();
    });

    server.await.unwrap();
    client.await.unwrap();
    client_mux.abort();
    server_mux.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn keep_alive_and_peer_sharing_are_independent() {
    let (client_bearer, server_bearer) = tokio::io::duplex(64);
    let (client_handles, client_mux) = network::mux::mux::<NodeToNode, _>(client_bearer);
    let (server_handles, server_mux) = network::mux::mux::<NodeToNode, _>(server_bearer);
    let (_, _, _, _, (keep_alive_client, _), (peer_sharing_client, _)) = client_handles;
    let (_, _, _, _, (_, keep_alive_server), (_, peer_sharing_server)) = server_handles;

    let client = tokio::spawn(async move {
        let response = keep_alive_client
            .send(&node_to_node::keep_alive::KeepAlive { cookie: 42 })
            .await
            .unwrap();
        let (payload, _) = response.receive().await.unwrap();
        assert_eq!(payload.decode().unwrap().cookie, 42);

        let response = peer_sharing_client
            .send(&node_to_node::peer_sharing::Request { amount: 2 })
            .await
            .unwrap();
        let (payload, _) = response.receive().await.unwrap();
        assert_eq!(payload.decode().unwrap().peers.len(), 2);
    });
    let server = tokio::spawn(async move {
        let node_to_node::keep_alive::client::Message::KeepAlive(payload, response) =
            keep_alive_server.receive().await.unwrap()
        else {
            panic!("unexpected keep-alive message");
        };
        let request = payload.decode().unwrap();
        response
            .send(&node_to_node::keep_alive::Response {
                cookie: request.cookie,
            })
            .await
            .unwrap();

        let request = peer_sharing_server.receive().await.unwrap();
        let node_to_node::peer_sharing::idle::Message::Request(payload, response) = request else {
            panic!("unexpected peer-sharing message");
        };
        assert_eq!(payload.decode().unwrap().amount, 2);
        response
            .send(&node_to_node::peer_sharing::Share {
                peers: vec![
                    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3001)),
                    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3002)),
                ],
            })
            .await
            .unwrap();
    });

    server.await.unwrap();
    client.await.unwrap();
    client_mux.abort();
    server_mux.abort();
}

#[tokio::test(flavor = "current_thread")]
async fn chain_sync_reassembles_messages_across_mux_segments() {
    let (client_bearer, server_bearer) = tokio::io::duplex(1024);
    let (client_handles, client_mux) = network::mux::mux::<NodeToNode, _>(client_bearer);
    let (server_handles, server_mux) = network::mux::mux::<NodeToNode, _>(server_bearer);
    let (_, (client, _), ..) = client_handles;
    let (_, (_, server), ..) = server_handles;
    let points: Vec<_> = (0..2_000)
        .map(|slot| network::Point::Block {
            slot,
            hash: [slot as u8; 32],
        })
        .collect();

    let client = tokio::spawn(async move {
        let response = client
            .send(&node_to_node::chain_sync::idle::FindIntersect { points })
            .await
            .unwrap();
        let node_to_node::chain_sync::intersect::Message::Found(payload, _) =
            response.receive().await.unwrap()
        else {
            panic!("intersection was not found");
        };
        assert_eq!(payload.decode().unwrap().point, network::Point::Genesis);
    });
    let server = tokio::spawn(async move {
        let node_to_node::chain_sync::idle::Message::FindIntersect(payload, response) =
            server.receive().await.unwrap()
        else {
            panic!("unexpected chain-sync message");
        };
        assert_eq!(payload.decode().unwrap().points.len(), 2_000);
        response
            .send(&node_to_node::chain_sync::intersect::Found {
                point: network::Point::Genesis,
                tip: network::Tip::Genesis,
            })
            .await
            .unwrap();
    });

    server.await.unwrap();
    client.await.unwrap();
    client_mux.abort();
    server_mux.abort();
}
