//! Address.

use crate::crypto::{Blake2b224, Blake2b224Digest, DigestWriter, digest::Digest};
use sha3::Sha3_256;
use tinycbor::{Encode, Encoded, Encoder};
use tinycbor_derive::{CborLen, Decode, Encode};

mod payload;
pub use payload::Payload;

mod attributes;
pub use attributes::Attributes;

mod data;
pub use data::Data;

/// Byron Era address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Address {
    /// The payload of the address.
    #[cbor(with = "Encoded<Payload>")]
    pub payload: Payload,
    /// A crc32 checksum of the CBOR-encoded payload.
    pub checksum: u32,
}

impl Address {
    pub fn new(payload: Payload) -> Self {
        struct Crc32Writer(crc32fast::Hasher);
        impl embedded_io::ErrorType for Crc32Writer {
            type Error = core::convert::Infallible;
        }

        impl tinycbor::Write for Crc32Writer {
            fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
                self.0.update(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> Result<(), Self::Error> {
                Ok(())
            }
        }

        let mut encoder = Encoder(Crc32Writer(crc32fast::Hasher::new()));
        payload.encode(&mut encoder);
        Self {
            payload,
            checksum: encoder.0.0.finalize(),
        }
    }
}

/// A Byron era address root.
///
/// This is used to compute the address root digest: the hash of this CBOR-encoded structure.
#[derive(Encode)]
pub struct Root<'a> {
    pub address_type: Type,
    pub data: Data<'a>,
    pub attributes: &'a Attributes,
}

/// The type of an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub enum Type {
    /// The address is a verifying key address.
    #[n(0)]
    VerifyingKey,
    /// The address is a redeem address.
    ///
    /// These were distributed to pre-sale Ada buyers from the Ada Voucher Vending Machine (AVVM)
    /// program.
    #[n(2)]
    Redeem,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tinycbor::{Decode, Decoder, Encode, Encoder};

    const TEST_VECTORS: [&str; 2] = [
        // From https://cardano-foundation.github.io/cardano-wallet/design/concepts/byron-address-format.html
        "Ae2tdPwUPEZLs4HtbuNey7tK4hTKrwNwYtGqp7bDfCy2WdR3P6735W5Yfpe",
        // From https://github.com/txpipe/pallas/blob/main/pallas-addresses/src/byron.rs
        "DdzFFzCqrht7PQiAhzrn6rNNoADJieTWBt8KeK9BZdUsGyX9ooYD9NpMCTGjQoUKcHN47g8JMXhvKogsGpQHtiQ65fZwiypjrC6d3a4Q",
    ];

    #[test]
    fn roundtrip_base58() {
        for vector in TEST_VECTORS {
            let cbor = bs58::decode(vector).into_vec().unwrap();
            let addr = Address::decode(&mut Decoder(&cbor)).unwrap();
            let mut encoder = Encoder(Vec::new());
            addr.encode(&mut encoder);
            let ours = bs58::encode(encoder.0).into_string();
            assert_eq!(vector, ours);
        }
    }
}
