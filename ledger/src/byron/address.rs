use tinycbor::Encoded;
use tinycbor_derive::{CborLen, Decode, Encode};

pub mod payload;
pub use payload::Payload;

pub mod distribution;
pub use distribution::Distribution;

pub mod attributes;
pub use attributes::Attributes;

pub mod data;
pub use data::Data;

/// Byron Era address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Address {
    #[cbor(with = "Encoded<Payload>")]
    pub payload: Payload,
    pub checksum: u32,
}

impl Address {
    pub fn new(payload: Payload) -> Self {
        // We know this cannot error because of Vec.
        let cbor_payload = tinycbor::to_vec(&payload);
        let checksum = crc32fast::hash(&cbor_payload);
        Self { payload, checksum }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tinycbor::{Decode, Decoder, Encode, Encoder};

    const TEST_VECTORS: [&str; 3] = [
        // From https://cardano-foundation.github.io/cardano-wallet/design/concepts/byron-address-format.html
        "37btjrVyb4KDXBNC4haBVPCrro8AQPHwvCMp3RFhhSVWwfFmZ6wwzSK6JK1hY6wHNmtrpTf1kdbva8TCneM2YsiXT7mrzT21EacHnPpz5YyUdj64na",
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
