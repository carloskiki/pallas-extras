//! Address.

use crate::crypto::{Blake2b224, Blake2b224Digest, DigestWriter, digest::Digest};
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
    #[cbor(with = "Encoded<Payload>")]
    pub payload: Payload,
    pub checksum: u32,
}

impl Address {
    pub fn new(payload: Payload) -> Self {
        let cbor_payload = tinycbor::to_vec(&payload);
        let checksum = crc32fast::hash(&cbor_payload);
        Self { payload, checksum }
    }
}

pub fn root_digest(
    address_type: Type,
    data: Data<'_>,
    attributes: &Attributes,
) -> Blake2b224Digest {
    #[derive(Encode)]
    struct Root<'a> {
        address_type: Type,
        data: Data<'a>,
        attributes: &'a Attributes,
    }

    let mut encoder = Encoder(DigestWriter(Blake2b224::default()));
    Root {
        address_type,
        data,
        attributes,
    }
    .encode(&mut encoder);

    encoder.0.0.finalize().into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub enum Type {
    #[n(0)]
    VerifyingKey,
    #[n(2)]
    Redeem,
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
