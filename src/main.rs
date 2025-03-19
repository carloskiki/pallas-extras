use std::{
    io::{Read, Write},
    net::TcpStream,
};

use minicbor::{data::Token, to_vec, Decode, Encode};
use ponk::network::{
    self, Header, NetworkMagic, NodeToClient,
    handshake::{self, VersionTable},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = std::time::Instant::now();
    let mut stream = TcpStream::connect("preview-node.play.dev.cardano.org:3001")?;

    let propose_versions = handshake::ClientMessage::ProposeVersions(VersionTable {
        versions: vec![(
            32788,
            handshake::NodeToClientVersionData {
                network_magic: NetworkMagic::Preview,
                query: true,
            },
        )],
    });
    let encoded = to_vec(&propose_versions)?;
    let header = network::Header {
        timestamp: now.elapsed().as_secs() as u32,
        protocol: network::Protocol {
            responder: false,
            protocol: NodeToClient::Handshake,
        },
        payload_len: encoded.len() as u16,
    };

    stream.write_all(&<[u8; 8]>::from(header))?;
    stream.write_all(&encoded)?;

    let mut buf = [0; 1024];
    stream.read_exact(&mut buf[0..8])?;
    let header = network::Header::<NodeToClient>::try_from(<[u8; 8]>::try_from(&buf[0..8])?)?;
    dbg!(&header);
    stream.read_exact(&mut buf[8..8 + header.payload_len as usize])?;
    let message = handshake::ServerMessage::<'_, handshake::NodeToClientVersionData>::decode(
        &mut minicbor::Decoder::new(&buf[8..8 + header.payload_len as usize]),
        &mut (),
    )?;
    dbg!(&message);

    Ok(())
}
