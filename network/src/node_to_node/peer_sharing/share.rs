use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use tinycbor::{
    CborLen, Decode, Encode,
    container::{self, bounded},
    tag,
};
use tinycbor_derive::{CborLen, Decode, Encode};
use zerocopy::transmute;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Share {
    pub peers: Vec<SocketAddr>,
}

impl Encode for Share {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        e.array(self.peers.len())?;
        for peer in self.peers.iter() {
            SocketCodec::from(peer).encode(e)?;
        }
        Ok(())
    }
}

impl Decode<'_> for Share {
    type Error = container::Error<bounded::Error<tag::Error<Error>>>;

    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        let mut visitor = d.array_visitor()?;
        let mut peers = Vec::with_capacity(visitor.remaining().unwrap_or(0));

        while let Some(peer_codec) = visitor.visit::<SocketCodec>() {
            let peer_codec = peer_codec?;
            peers.push(SocketAddr::from(peer_codec));
        }
        Ok(Share { peers })
    }
}

impl CborLen for Share {
    fn cbor_len(&self) -> usize {
        1 + self
            .peers
            .iter()
            .map(|peer| SocketCodec::from(peer).cbor_len())
            .sum::<usize>()
    }
}

#[derive(Encode, Decode, CborLen)]
enum SocketCodec {
    #[n(0)]
    V4(u32, u16),
    #[n(1)]
    V6(u32, u32, u32, u32, u16),
}

impl From<&SocketAddr> for SocketCodec {
    fn from(addr: &SocketAddr) -> Self {
        match addr {
            // TODO: check byte order for IP addresses (both v4 and v6)
            SocketAddr::V4(addr) => SocketCodec::V4(addr.ip().to_bits(), addr.port()),
            SocketAddr::V6(addr) => {
                let [a, b, c, d]: [[u8; 4]; 4] = transmute!(addr.ip().octets());
                SocketCodec::V6(
                    u32::from_be_bytes(a),
                    u32::from_be_bytes(b),
                    u32::from_be_bytes(c),
                    u32::from_be_bytes(d),
                    addr.port(),
                )
            }
        }
    }
}

impl From<SocketCodec> for SocketAddr {
    fn from(codec: SocketCodec) -> Self {
        match codec {
            SocketCodec::V4(ip, port) => {
                SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from_bits(ip), port))
            }
            SocketCodec::V6(octets0, octets1, octets2, octets3, port) => {
                let ip: Ipv6Addr = Ipv6Addr::from_bits(transmute!([[
                    octets0.to_be_bytes(),
                    octets1.to_be_bytes(),
                    octets2.to_be_bytes(),
                    octets3.to_be_bytes()
                ]]));
                SocketAddr::V6(SocketAddrV6::new(ip, port, 0, 0))
            }
        }
    }
}

impl crate::Message for Share {
    const TAG: u64 = 1;

    type ToState = super::Idle;
}
