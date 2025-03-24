use ed25519_dalek::Signature;
use minicbor::{Decode, Encode};

use crate::{Blake2b224Digest, ExtendedVerifyingKey, shelley::{plutus, protocol}};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cbor(map)]
pub struct Set {
    #[n(0)]
    #[cbor(with = "crate::cbor::boxed_slice", has_nil)]
    pub verifying_keys: Box<[VerifyingKey]>,
    #[n(1)]
    #[cbor(with = "crate::cbor::boxed_slice", has_nil)]
    pub native_scripts: Box<[Script]>,
    #[n(2)]
    #[cbor(with = "crate::cbor::boxed_slice", has_nil)]
    pub bootstraps: Box<[Bootstrap]>,
    #[n(3)]
    #[cbor(with = "crate::cbor::boxed_slice", has_nil)]
    pub plutus_v1: Box<[plutus::Script]>,
    #[n(4)]
    #[cbor(with = "crate::cbor::boxed_slice", has_nil)]
    pub plutus_data: Box<[plutus::Data]>,
    #[n(5)]
    #[cbor(with = "crate::cbor::boxed_slice", has_nil)]
    pub redeemers: Box<[Redeemer]>,
    #[n(6)]
    #[cbor(with = "crate::cbor::boxed_slice", has_nil)]
    pub plutus_v2: Box<[plutus::Script]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct VerifyingKey {
    #[n(0)]
    #[cbor(with = "crate::cbor::compressed_edwards_y")]
    pub vkey: crate::VerifyingKey,
    #[n(1)]
    #[cbor(with = "crate::cbor::signature")]
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[cbor(flat)]
pub enum Script {
    #[n(0)]
    Vkey(#[n(0)] Blake2b224Digest),
    #[n(1)]
    All(#[cbor(n(0), with = "crate::cbor::boxed_slice")] Box<[Script]>),
    #[n(2)]
    Any(#[cbor(n(0), with = "crate::cbor::boxed_slice")] Box<[Script]>),
    #[n(3)]
    NofK(
        #[n(0)] u64,
        #[cbor(n(1), with = "crate::cbor::boxed_slice")] Box<[Script]>,
    ),
    #[n(4)]
    InvalidBefore(#[n(0)] u64),
    #[n(5)]
    InvalidHereafter(#[n(0)] u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bootstrap {
    pub key: ExtendedVerifyingKey,
    pub signature: Signature,
    // TODO: should we ignore the attributes?
    pub attributes: Box<[u8]>,
}

impl<C> Encode<C> for Bootstrap {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(4)?;
        crate::cbor::compressed_edwards_y::encode(&self.key.key, e, &mut ())?;
        crate::cbor::signature::encode(&self.signature, e, &mut ())?;

        e.bytes(&self.key.chain_code)?.bytes(&self.attributes)?.ok()
    }
}

impl<C> Decode<'_, C> for Bootstrap {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let array_size = d.array()?;
        if !matches!(array_size, Some(4)) {
            return Err(minicbor::decode::Error::message("Invalid array size"));
        }
        use minicbor::bytes::ByteArray;
        
        let compressed_y = crate::cbor::compressed_edwards_y::decode(d, &mut ())?;
        let signature = crate::cbor::signature::decode(d, &mut ())?;
        let chain_code: ByteArray<32> = d.decode()?;
        let attributes = crate::cbor::boxed_bytes::decode(d, &mut ())?;

        let key = ExtendedVerifyingKey {
            key: compressed_y,
            chain_code: *chain_code,
        };

        Ok(Bootstrap {
            key,
            signature,
            attributes,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct Redeemer {
    #[n(0)]
    pub tag: Tag,
    #[n(1)]
    pub index: u64,
    #[n(2)]
    pub data: plutus::Data,
    #[n(3)]
    pub execution_units: protocol::ExecutionUnits,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[cbor(index_only)]
pub enum Tag {
    #[n(0)]
    Spend,
    #[n(1)]
    Mint,
    // TODO: find name for this
    #[n(2)]
    Cert,
    #[n(3)]
    Reward,
}
