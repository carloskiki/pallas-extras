use std::{
    io::{Read, Write},
    net::TcpStream,
};

use network::{
    self, NetworkMagic, Tip, chain_sync,
    handshake::{self, NodeToNodeVersionData, VersionTable},
    mux::header::{Header, ProtocolNumber},
    protocol::NodeToNode,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = std::time::Instant::now();
    let mut stream = TcpStream::connect("preview-node.play.dev.cardano.org:3001")?;

    // Handshake

    let message = handshake::ClientMessage::ProposeVersions(VersionTable {
        versions: vec![(
            14,
            handshake::NodeToNodeVersionData {
                network_magic: NetworkMagic::Preview,
                diffusion_mode: false,
                peer_sharing: false,
                query: false,
            },
        )],
    });
    let payload = minicbor::to_vec(&message)?;
    let header = network::mux::header::Header {
        timestamp: now.elapsed().as_micros() as u32,
        protocol: ProtocolNumber {
            protocol: network::protocol::NodeToNode::Handshake,
            server: false,
        },
        payload_len: payload.len() as u16,
    };
    let header_bytes: [u8; 8] = header.into();

    stream.write_all(&header_bytes)?;
    stream.write_all(&payload)?;

    let mut header_buf = [0; 8];
    stream.read_exact(&mut header_buf)?;
    let header: Header<NodeToNode> = Header::try_from(header_buf)?;
    let mut payload_buf = [0; 4096];
    stream.read_exact(&mut payload_buf[..header.payload_len as usize])?;
    let msg: handshake::ServerMessage<NodeToNodeVersionData> =
        minicbor::decode(&payload_buf[..header.payload_len as usize])?;
    dbg!(msg);

    // Chain Sync first request

    let msg = chain_sync::ClientMessage::Next;
    let payload = minicbor::to_vec(&msg)?;
    let header = network::mux::header::Header {
        timestamp: now.elapsed().as_micros() as u32,
        protocol: ProtocolNumber {
            protocol: network::protocol::NodeToNode::ChainSync,
            server: false,
        },
        payload_len: payload.len() as u16,
    };

    let header_bytes: [u8; 8] = header.into();
    stream.write_all(&header_bytes)?;
    stream.write_all(&payload)?;

    let mut header_buf = [0; 8];
    stream.read_exact(&mut header_buf)?;
    let header: Header<NodeToNode> = Header::try_from(header_buf)?;
    let mut payload_buf = [0; 4096];
    stream.read_exact(&mut payload_buf[..header.payload_len as usize])?;
    let msg: chain_sync::ServerMessage =
        minicbor::decode(&payload_buf[..header.payload_len as usize])?;

    let chain_sync::ServerMessage::RollBackward {
        tip: Tip::Block { slot, hash, .. },
        ..
    } = msg
    else {
        panic!("Expected RollBackward, got {:?}", msg);
    };

    // Chain Sync find intersection with latest block

    let msg = chain_sync::ClientMessage::FindIntersect {
        points: vec![network::Point::Block { slot, hash }].into_boxed_slice(),
    };
    let payload = minicbor::to_vec(&msg)?;
    let header = network::mux::header::Header {
        timestamp: now.elapsed().as_micros() as u32,
        protocol: ProtocolNumber {
            protocol: network::protocol::NodeToNode::ChainSync,
            server: false,
        },
        payload_len: payload.len() as u16,
    };

    let header_bytes: [u8; 8] = header.into();
    stream.write_all(&header_bytes)?;
    stream.write_all(&payload)?;

    let mut header_buf = [0; 8];
    stream.read_exact(&mut header_buf)?;
    let header: Header<NodeToNode> = Header::try_from(header_buf)?;
    let mut payload_buf = [0; 4096];
    stream.read_exact(&mut payload_buf[..header.payload_len as usize])?;

    let msg: chain_sync::ServerMessage =
        minicbor::decode(&payload_buf[..header.payload_len as usize])?;
    dbg!(msg);

    // Chain Sync with upstream

    let msg = chain_sync::ClientMessage::Next;
    let payload = minicbor::to_vec(&msg)?;
    let header = network::mux::header::Header {
        timestamp: now.elapsed().as_micros() as u32,
        protocol: ProtocolNumber {
            protocol: network::protocol::NodeToNode::ChainSync,
            server: false,
        },
        payload_len: payload.len() as u16,
    };

    let header_bytes: [u8; 8] = header.into();
    stream.write_all(&header_bytes)?;
    stream.write_all(&payload)?;

    let mut header_buf = [0; 8];
    stream.read_exact(&mut header_buf)?;
    let header: Header<NodeToNode> = Header::try_from(header_buf)?;
    let mut payload_buf = [0; 4096];
    stream.read_exact(&mut payload_buf[..header.payload_len as usize])?;
    let msg: chain_sync::ServerMessage =
        minicbor::decode(&payload_buf[..header.payload_len as usize])?;

    dbg!(msg);
    
    // Setup Await Reply
    
    stream.write_all(&header_bytes)?;
    stream.write_all(&payload)?;

    stream.read_exact(&mut header_buf)?;
    let header: Header<NodeToNode> = Header::try_from(header_buf)?;
    stream.read_exact(&mut payload_buf[..header.payload_len as usize])?;
    let msg: chain_sync::ServerMessage =
        minicbor::decode(&payload_buf[..header.payload_len as usize])?;
    
    dbg!(msg, header);

    stream.read_exact(&mut header_buf)?;
    let header: Header<NodeToNode> = Header::try_from(header_buf)?;
    stream.read_exact(&mut payload_buf[..header.payload_len as usize])?;
    dbg!(header);

    Ok(())
}
