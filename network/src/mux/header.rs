use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, network_endian::{U16, U32}};

/// The lower 32 bits of the peer's monotonic microseconds clock.
#[derive(Debug, Clone, Copy, Default, FromBytes, IntoBytes, Immutable)]
#[repr(transparent)]
pub struct Timestamp(pub U32);

impl Timestamp {
    pub fn elapsed(time: &std::time::Instant) -> Self {
        Self((time.elapsed().as_micros() as u32).into())
    }
}

// TODO: use network order for everything.
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Header {
    pub timestamp: Timestamp,
    pub protocol: ProtocolNumber,
    pub payload_len: U16,
}

#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable)]
#[repr(transparent)]
pub struct ProtocolNumber(U16);

impl ProtocolNumber {
    pub fn new(protocol: u16, server_sent: bool) -> Self {
        Self((protocol | (server_sent as u16 * 0x8000)).into())
    }

    pub fn number(&self) -> u16 {
        u16::from(self.0) & 0x7FFF
    }

    pub fn server_sent(&self) -> bool {
        self.0 & 0x8000 != 0
    }
}
