use curve25519_dalek::edwards::CompressedEdwardsY;
use digest::{consts::U28, Digest};
use minicbor::{
    bytes::{ByteArray, ByteVec},
    data::IanaTag,
    decode,
    encode::{self, Write},
    Decode, Decoder, Encode, Encoder,
};
use sha3::Sha3_256;

use crate::{ExtendedVerifyingKey, VerifyingKey};

pub type Blake2b224 = blake2::Blake2b<U28>;
type Blake2b224Digest = ByteArray<28>;

pub struct Address {
    payload: Payload,
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
            .encode(&root.addr_type)
            .unwrap()
            .encode(&root.spending_data)
            .unwrap()
            .encode(&attributes)
            .unwrap();

        let root_bytes = root_encoder.into_writer();
        let root_digest: Blake2b224Digest = ByteArray::from(<[u8; 28]>::from(Blake2b224::digest(
            Sha3_256::digest(&root_bytes),
        )));
        let payload = Payload {
            root_digest,
            attributes,
            addr_type: root.addr_type,
        };
        // We know this cannot error because of Vec.
        let cbor_payload = minicbor::to_vec(&payload).unwrap();
        let crc = crc32fast::hash(&cbor_payload);
        Self { payload, crc }
    }

    pub fn from_base58(s: &str) -> Result<Self, FromBase58Error> {
        let bytes = bs58::decode(s).into_vec()?;
        let addr = minicbor::decode(&bytes)?;
        Ok(addr)
    }

    pub fn to_base58(&self) -> String {
        // We know this cannot error because of Vec.
        let bytes = minicbor::to_vec(self).unwrap();
        bs58::encode(bytes).into_string()
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

impl<C> Encode<C> for Address {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        e.array(2)?
            .tag(IanaTag::Cbor)?
            .bytes(&minicbor::to_vec(&self.payload).unwrap())?
            .u32(self.crc)?
            .ok()
    }
}

impl<C> Decode<'_, C> for Address {
    fn decode(d: &mut Decoder<'_>, _: &mut C) -> Result<Self, decode::Error> {
        let array_len = d.array()?;
        if array_len != Some(2) {
            return Err(decode::Error::message("invalid Address cbor length"));
        }

        let tag = d.tag()?;
        if tag != IanaTag::Cbor.tag() {
            return Err(decode::Error::message("invalid Address Payload tag"));
        }

        let bytes = d.bytes()?;
        let payload: Payload = minicbor::decode(bytes)?;
        let crc = d.u32()?;
        Ok(Self { payload, crc })
    }
}

#[derive(Encode, Decode)]
struct Payload {
    #[n(0)]
    root_digest: Blake2b224Digest,
    #[n(1)]
    attributes: Attributes,
    #[n(2)]
    addr_type: AddressType,
}

pub struct Root {
    addr_type: AddressType,
    spending_data: SpendingData,
}

enum StakeDistribution {
    Bootstrap,
    SingleKey(Blake2b224Digest),
}

impl<C> Encode<C> for StakeDistribution {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        match self {
            StakeDistribution::SingleKey(x) => {
                e.array(2)?;
                e.u32(0)?;
                e.encode(x)?;
            }
            StakeDistribution::Bootstrap => {
                e.array(1)?;
                e.u32(1)?;
            }
        }
        Ok(())
    }
}

impl<C> Decode<'_, C> for StakeDistribution {
    fn decode(d: &mut Decoder<'_>, _: &mut C) -> Result<Self, decode::Error> {
        d.array()?;
        let variant = d.u32()?;

        match variant {
            0 => Ok(StakeDistribution::SingleKey(d.decode()?)),
            1 => Ok(StakeDistribution::Bootstrap),
            _ => Err(minicbor::decode::Error::message(
                "Invalid StakeDistribution variant",
            )),
        }
    }
}

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

impl<C> Encode<C> for AddressType {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        e.u32(self.0).map(|_| ())
    }
}

impl<C> Decode<'_, C> for AddressType {
    fn decode(d: &mut Decoder<'_>, _: &mut C) -> Result<Self, decode::Error> {
        Ok(AddressType(d.u32()?))
    }
}

enum SpendingData {
    PublicKey(ExtendedVerifyingKey),
    Redeem(VerifyingKey),
}

impl<C> Encode<C> for SpendingData {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        match self {
            SpendingData::PublicKey(x) => {
                e.array(2)?;
                e.u32(0)?;
                e.encode(x)?;
            }
            SpendingData::Redeem(x) => {
                e.array(2)?;
                e.u32(1)?;
                e.encode(x.as_bytes())?;
            }
        }
        Ok(())
    }
}

impl<C> Decode<'_, C> for SpendingData {
    fn decode(d: &mut Decoder<'_>, _: &mut C) -> Result<Self, decode::Error> {
        d.array()?;
        let discriminant = d.u32()?;
        Ok(match discriminant {
            0 => SpendingData::PublicKey(d.decode()?),
            1 => {
                let bytes: [u8; 32] = d.bytes()?.try_into().map_err(decode::Error::custom)?;
                SpendingData::Redeem(CompressedEdwardsY(bytes))
            }
            _ => return Err(decode::Error::message("invalid SpendingData discriminant")),
        })
    }
}

pub struct Attributes {
    distribution: Option<StakeDistribution>,
    // Only to retain information when encoding, we do not use this.
    _key_derivation_path: Option<ByteArray<30>>,
    network_magic: Option<u32>,
}

impl<C> minicbor::Encode<C> for Attributes {
    fn encode<W>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> core::result::Result<(), minicbor::encode::Error<W::Error>>
    where
        W: Write,
    {
        let field_count = self.distribution.is_some() as u64
            + self._key_derivation_path.is_some() as u64
            + self.network_magic.is_some() as u64;
        e.map(field_count)?;
        if let Some(distribution) = &self.distribution {
            e.u32(0)?;
            e.encode(distribution)?;
        }
        if let Some(key_derivation_path) = &self._key_derivation_path {
            e.u32(1)?;
            e.encode(key_derivation_path)?;
        }
        if let Some(network_magic) = &self.network_magic {
            e.u32(2)?;
            e.encode(ByteVec::from(minicbor::to_vec(network_magic).unwrap()))?;
        }
        Ok(())
    }
}
impl<'bytes, Ctx> minicbor::Decode<'bytes, Ctx> for Attributes {
    fn decode(
        d: &mut minicbor::Decoder<'bytes>,
        _: &mut Ctx,
    ) -> core::result::Result<Attributes, minicbor::decode::Error> {
        let mut distribution: Option<StakeDistribution> = None;
        let mut key_derivation_path: Option<ByteArray<30>> = None;
        let mut network_magic: Option<u32> = None;
        if let Some(map_len) = d.map()? {
            for _ in 0..map_len {
                match d.u32()? {
                    0 => distribution = d.decode()?,
                    1 => key_derivation_path = d.decode()?,
                    2 => {
                        let bytes = d.bytes()?;
                        network_magic = Some(minicbor::decode(bytes)?);
                    }
                    _ => d.skip()?,
                }
            }
        } else {
            while minicbor::data::Type::Break != d.datatype()? {
                match d.u32()? {
                    0 => distribution = d.decode()?,
                    1 => key_derivation_path = d.decode()?,
                    2 => {
                        let bytes = d.bytes()?;
                        network_magic = Some(minicbor::decode(bytes)?);
                    }
                    _ => d.skip()?,
                }
            }
            d.skip()?
        }
        Ok(Attributes {
            distribution,
            _key_derivation_path: key_derivation_path,
            network_magic,
        })
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
