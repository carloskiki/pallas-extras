use crate::traits::protocol::Protocol;
use zerocopy::transmute;

/// The lower 32 bits of the peer's monotonic clock, representing microseconds.
#[derive(Debug, Clone, Copy)]
pub struct Timestamp(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct Header<T> {
    pub timestamp: Timestamp,
    pub protocol: ProtocolNumber<T>,
    pub payload_len: u16,
}

impl<P> TryFrom<[u8; 8]> for Header<P>
where
    P: Protocol,
{
    type Error = u16;

    fn try_from(value: [u8; 8]) -> std::result::Result<Self, u16> {
        let [timestamp, rest]: [[u8; 4]; 2] = transmute!(value);
        let [protocol, payload_len]: [[u8; 2]; 2] = transmute!(rest);

        let timestamp = u32::from_be_bytes(timestamp);
        let protocol = ProtocolNumber::<P>::try_from(u16::from_be_bytes(protocol))?;
        let payload_len = u16::from_be_bytes(payload_len);

        Ok(Self {
            timestamp: Timestamp(timestamp),
            protocol,
            payload_len,
        })
    }
}

impl<P> From<Header<P>> for [u8; 8]
where
    P: Protocol,
{
    fn from(value: Header<P>) -> Self {
        let protocol_value: u16 = value.protocol.into();
        let protocol_and_payload_len: [u8; 4] = transmute!([
            protocol_value.to_be_bytes(),
            value.payload_len.to_be_bytes()
        ]);
        let timestamp: [u8; 4] = value.timestamp.0.to_be_bytes();

        transmute!([timestamp, protocol_and_payload_len])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProtocolNumber<T> {
    pub protocol: T,
    pub server_sent: bool,
}

impl<P> TryFrom<u16> for ProtocolNumber<P>
where
    P: Protocol,
{
    type Error = u16;

    fn try_from(value: u16) -> std::result::Result<Self, u16> {
        let responder = value & 0x8000 != 0;
        let value = value & 0x7FFF;
        let protocol = P::from_number(value).ok_or(value)?;

        Ok(Self {
            server_sent: responder,
            protocol,
        })
    }
}

impl<P> From<ProtocolNumber<P>> for u16
where
    P: Protocol,
{
    fn from(value: ProtocolNumber<P>) -> Self {
        let responder = if value.server_sent { 0x8000 } else { 0 };
        let protocol: u16 = value.protocol.number();

        responder | protocol
    }
}
