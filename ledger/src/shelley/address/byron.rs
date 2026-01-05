use digest::Digest;
use minicbor::{
    CborLen, Decode, Encode, Encoder,
    bytes::ByteArray,
};
use sha3::Sha3_256;

use crate::crypto::{Blake2b224, Blake2b224Digest, VerifyingKey};
use bip32::ExtendedVerifyingKey;


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Address {
    #[cbor(n(0), with = "cbor_util::cbor_encoded")]
    payload: Payload,
    #[n(1)]
    crc: u32,
}

impl Address {
    pub fn new(root: Root, attributes: Attributes) -> Self {
        // Arbitrary size that should fit most encodings without resizing
        let mut root_encoder = Encoder::new(Vec::with_capacity(128));
        // Unwrap because we know the writer (Vec) can't fail
        root_encoder
            .array(3)
            .unwrap()
            .encode(root.addr_type)
            .unwrap()
            .encode(root.spending_data)
            .unwrap()
            .encode(&attributes)
            .unwrap();

        let root_bytes = root_encoder.into_writer();
        let root_digest: Blake2b224Digest =
            Blake2b224::digest(Sha3_256::digest(&root_bytes)).into();
        let payload = Payload {
            root_digest,
            attributes,
            addr_type: root.addr_type,
        };
        // We know this cannot error because of Vec.
        let cbor_payload = minicbor::to_vec(&payload)
            .expect("should not error because the writer is a vec (which has Infallibe error)");
        let crc = crc32fast::hash(&cbor_payload);
        Self { payload, crc }
    }

    pub fn from_base58(s: &str) -> Result<Self, FromBase58Error> {
        let bytes = bs58::decode(s).into_vec()?;
        let addr = minicbor::decode(&bytes)?;
        Ok(addr)
    }

    pub fn to_base58(&self) -> String {
        let bytes = minicbor::to_vec(self)
            .expect("should not error because the writer is a vec (which has Infallibe error)");
        bs58::encode(bytes).into_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
struct Payload {
    #[cbor(n(0), with = "minicbor::bytes")]
    root_digest: Blake2b224Digest,
    #[n(1)]
    attributes: Attributes,
    #[n(2)]
    addr_type: AddressType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Root {
    addr_type: AddressType,
    spending_data: SpendingData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(flat)]
enum StakeDistribution {
    #[n(1)]
    Bootstrap,
    #[n(0)]
    SingleKey(#[cbor(n(0), with = "minicbor::bytes")] Blake2b224Digest),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(transparent)]
pub struct AddressType(pub u32);

impl AddressType {
    pub fn is_key(&self) -> bool {
        self.0 == 0
    }

    pub fn is_script(&self) -> bool {
        self.0 == 1
    }

    pub fn is_redeem(&self) -> bool {
        self.0 == 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, CborLen)]
#[cbor(flat)]
enum SpendingData {
    #[n(0)]
    PublicKey(#[n(0)] ExtendedVerifyingKey),
    #[n(1)]
    Redeem(#[cbor(n(0), with = "minicbor::bytes")] VerifyingKey),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Attributes {
    #[n(0)]
    distribution: Option<StakeDistribution>,
    // Only to retain information when encoding, we do not use this.
    #[n(1)]
    #[cbor(with = "cbor_util::bytes")]
    _key_derivation_path: Option<Box<[u8]>>,
    #[cbor(n(2), with = "cbor_no_tag", has_nil)]
    network_magic: Option<u32>,
}

mod cbor_no_tag {
    use minicbor::{CborLen, Decoder, Encoder, decode as de, encode as en};

    pub fn encode<C, W: en::Write>(
        value: &Option<u32>,
        e: &mut Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        e.bytes_len(value.cbor_len(ctx) as u64)?.encode_with(value, ctx)?.ok()
    }

    pub fn decode<C>(d: &mut Decoder<'_>, ctx: &mut C) -> Result<Option<u32>, de::Error> {
        let store;
        let bytes;
        match d.datatype()? {
            minicbor::data::Type::Bytes => {
                bytes = d.bytes()?;
            }
            minicbor::data::Type::BytesIndef => {
                store = cbor_util::bytes_iter_collect(d.bytes_iter()?)?;
                bytes = &store;
            }
            t => return Err(de::Error::type_mismatch(t).at(d.position())),
        }

        let mut inner_decoder = Decoder::new(bytes);
        inner_decoder.decode_with(ctx)
    }

    pub fn cbor_len<C>(value: &Option<u32>, ctx: &mut C) -> usize {
        let bytes_len = value.cbor_len(ctx);
        bytes_len.cbor_len(ctx) + bytes_len
    }

    pub fn nil() -> Option<Option<u32>> {
        Some(None)
    }

    pub fn is_nil(value: &Option<u32>) -> bool {
        value.is_none()
    }
}

#[derive(Debug)]
pub enum FromBase58Error {
    Base58(bs58::decode::Error),
    Decode(minicbor::decode::Error),
}

impl From<bs58::decode::Error> for FromBase58Error {
    fn from(e: bs58::decode::Error) -> Self {
        FromBase58Error::Base58(e)
    }
}

impl From<minicbor::decode::Error> for FromBase58Error {
    fn from(e: minicbor::decode::Error) -> Self {
        FromBase58Error::Decode(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_VECTORS: [&str; 3] = [
        "37btjrVyb4KDXBNC4haBVPCrro8AQPHwvCMp3RFhhSVWwfFmZ6wwzSK6JK1hY6wHNmtrpTf1kdbva8TCneM2YsiXT7mrzT21EacHnPpz5YyUdj64na",
        "Ae2tdPwUPEZLs4HtbuNey7tK4hTKrwNwYtGqp7bDfCy2WdR3P6735W5Yfpe",
        "DdzFFzCqrht7PQiAhzrn6rNNoADJieTWBt8KeK9BZdUsGyX9ooYD9NpMCTGjQoUKcHN47g8JMXhvKogsGpQHtiQ65fZwiypjrC6d3a4Q",
    ];

    #[test]
    fn roundtrip_base58() {
        for vector in TEST_VECTORS {
            let addr = Address::from_base58(vector).unwrap();
            let ours = addr.to_base58();
            assert_eq!(vector, ours);
        }
    }
}
