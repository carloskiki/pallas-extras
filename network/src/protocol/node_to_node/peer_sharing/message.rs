use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use minicbor::{Decode, Encode};
use zerocopy::transmute;

use crate::{traits::message::{nop_codec, Message}, typefu::coproduct::Coprod};

use super::state::{Busy, Idle};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[cbor(transparent)]
pub struct ShareRequest {
    pub amount: u8,
}

impl Message for ShareRequest {
    const SIZE_LIMIT: usize = 5760;

    const TAG: u8 = 0;

    const ELEMENT_COUNT: u64 = 1;

    type ToState = Busy;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SharePeers {
    pub peers: Box<[SocketAddr]>,
}

impl Message for SharePeers {
    const SIZE_LIMIT: usize = 5760;

    const TAG: u8 = 1;

    const ELEMENT_COUNT: u64 = 1;

    type ToState = Idle;
}

impl<C> Encode<C> for SharePeers {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(self.peers.len() as u64)?;
        for peer in self.peers.iter() {
            match peer {
                SocketAddr::V4(socket_addr_v4) => {
                    e.array(3)?
                        .u8(0)?
                        .u32(u32::from_ne_bytes(socket_addr_v4.ip().octets()))?
                        .u16(socket_addr_v4.port())?;
                }
                SocketAddr::V6(socket_addr_v6) => {
                    let [octets0, octets1, octets2, octets3]: [u32; 4] =
                        transmute!(socket_addr_v6.ip().octets());
                    e.array(6)?
                        .u8(1)?
                        .u32(octets0)?
                        .u32(octets1)?
                        .u32(octets2)?
                        .u32(octets3)?
                        .u16(socket_addr_v6.port())?;
                }
            }
        }
        Ok(())
    }
}

impl<C> Decode<'_, C> for SharePeers {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let len = d.array()?;
        let mut peers = Vec::with_capacity(len.unwrap_or(4) as usize);

        let mut index = 0;
        while len.is_none_or(|l| l != index) && d.datatype()? != minicbor::data::Type::Break {
            index += 1;
            let peer_type = d.u8()?;
            match peer_type {
                0 => {
                    let ip = d.u32()?;
                    let port = d.u16()?;
                    peers.push(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from_bits(ip), port)));
                }
                1 => {
                    let octets0 = d.u32()?;
                    let octets1 = d.u32()?;
                    let octets2 = d.u32()?;
                    let octets3 = d.u32()?;
                    let port = d.u16()?;
                    let ip = Ipv6Addr::from_bits(transmute!([octets0, octets1, octets2, octets3]));
                    peers.push(SocketAddr::V6(SocketAddrV6::new(ip, port, 0, 0)));
                }
                _ => return Err(minicbor::decode::Error::message("Invalid peer type, expected 0 or 1")),
            }
        }
        if len.is_none() {
            d.skip()?;
        } 

        Ok(SharePeers {
            peers: peers.into_boxed_slice(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Done;

impl Message for Done {
    const SIZE_LIMIT: usize = 5760;

    const TAG: u8 = 3;

    const ELEMENT_COUNT: u64 = 0;

    type ToState = crate::traits::state::Done;
}

nop_codec!(Done);
