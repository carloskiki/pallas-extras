use std::{
    io::{Read, Write},
    net::TcpStream,
};

use ponk::network::{
    Header, NetworkMagic,
    handshake::{self, VersionTable},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = std::time::Instant::now();
    let mut stream = TcpStream::connect("preview-node.play.dev.cardano.org:3001")?;

    let propose_versions = handshake::ClientMessage::ProposeVersions(VersionTable {
        versions: vec![(
            14,
            handshake::VersionData {
                network_magic: NetworkMagic::Preview,
                diffusion_mode: false,
                peer_sharing: false,
                query: false,
            },
        )],
    });
    let payload = minicbor::to_vec(propose_versions)?;
    let header = Header {
        timestamp: now.elapsed().as_micros() as u32,
        protocol: 0,
        payload_len: payload.len() as u16,
    };
    let header_bytes: [u8; 8] = header.into();

    stream.write_all(&header_bytes)?;
    stream.write_all(&payload)?;

    let mut buf = [0; 256];
    let mut start = 0;
    let _bytes_read = stream.read(&mut buf[start..])?;
    
    let header_bytes: [u8; 8] = buf[0..8].try_into()?;
    let header = Header::from(header_bytes);
    
    let tokens: Vec<minicbor::data::Token> = minicbor::decode(&buf[8..])?;
    let response: handshake::ServerMessage = minicbor::decode(&buf[8..])?;
    dbg!(header, tokens, response);

    Ok(())
}
