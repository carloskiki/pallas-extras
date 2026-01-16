use super::wrapper;
use macro_rules_attribute::apply;
use tinycbor::{CborLen, Decode, Encode};

#[apply(wrapper)]
pub struct Ipv4Addr(pub Option<std::net::Ipv4Addr>);

impl Encode for Ipv4Addr {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        match &self.0 {
            Some(addr) => addr.octets().encode(e),
            None => tinycbor::primitive::Null.encode(e),
        }
    }
}

impl CborLen for Ipv4Addr {
    fn cbor_len(&self) -> usize {
        match &self.0 {
            Some(addr) => addr.octets().cbor_len(),
            None => tinycbor::primitive::Null.cbor_len(),
        }
    }
}

impl Decode<'_> for Ipv4Addr {
    type Error = <[u8; 4] as Decode<'static>>::Error;

    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        if d.datatype()? == tinycbor::Type::Null {
            d.next().expect("peeked").expect("null");
            return Ok(Ipv4Addr(None));
        }
        let octets: [u8; 4] = Decode::decode(d)?;
        Ok(Ipv4Addr(Some(std::net::Ipv4Addr::from(octets))))
    }
}

#[apply(wrapper)]
pub struct Ipv6Addr(pub Option<std::net::Ipv6Addr>);

impl Encode for Ipv6Addr {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        match &self.0 {
            Some(addr) => addr.octets().encode(e),
            None => tinycbor::primitive::Null.encode(e),
        }
    }
}

impl CborLen for Ipv6Addr {
    fn cbor_len(&self) -> usize {
        match &self.0 {
            Some(addr) => addr.octets().cbor_len(),
            None => tinycbor::primitive::Null.cbor_len(),
        }
    }
}

impl Decode<'_> for Ipv6Addr {
    type Error = <[u8; 16] as Decode<'static>>::Error;

    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        if d.datatype()? == tinycbor::Type::Null {
            d.next().expect("peeked").expect("null");
            return Ok(Ipv6Addr(None));
        }
        let octets: [u8; 16] = Decode::decode(d)?;
        Ok(Ipv6Addr(Some(std::net::Ipv6Addr::from(octets))))
    }
}
