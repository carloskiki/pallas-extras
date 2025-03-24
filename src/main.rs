use std::{
    io::{Read, Write},
    net::TcpStream,
};

use minicbor::{Decode, Encode, data::Token, to_vec};
use network::{
    self, handshake::{self, VersionTable}, Header, NetworkMagic, NodeToNode
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = std::time::Instant::now();
    let mut stream = TcpStream::connect("preview-node.play.dev.cardano.org:3001")?;

    // let propose_versions = handshake::ClientMessage::ProposeVersions(VersionTable {
    //     versions: vec![(
    //         14,
    //         handshake::NodeToNodeVersionData {
    //             network_magic: NetworkMagic::Preview,
    //             query: false,
    //             diffusion_mode: false,
    //             peer_sharing: false,
    //         },
    //     )],
    // });
    // 
    // let encoded = to_vec(&propose_versions)?;
    // let header = network::Header {
    //     timestamp: now.elapsed().as_micros() as u32,
    //     protocol: network::Protocol {
    //         responder: false,
    //         protocol: NodeToNode::Handshake,
    //     },
    //     payload_len: encoded.len() as u16,
    // };

    // stream.write_all(&<[u8; 8]>::from(header))?;
    // stream.write_all(&encoded)?;

    let mut buf = [0; 4096];
    stream.read_exact(&mut buf[0..8])?;
    let header = network::Header::<NodeToNode>::try_from(<[u8; 8]>::try_from(&buf[0..8])?)?;
    dbg!(&header);
    stream.read_exact(&mut buf[8..8 + header.payload_len as usize])?;
    let message = handshake::ClientMessage::<handshake::NodeToNodeVersionData>::decode(
        &mut minicbor::Decoder::new(&buf[8..8 + header.payload_len as usize]),
        &mut (),
    )?;
    dbg!(&message);

    Ok(())
}
