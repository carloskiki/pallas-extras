use bip32::ExtendedVerifyingKey;
use ed25519_dalek::Signature;
use minicbor::{Decode, Encode};

use crate::{crypto::Blake2b224Digest, plutus, protocol};

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
    #[cbor(with = "minicbor::bytes")]
    pub vkey: crate::crypto::VerifyingKey,
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
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(4)?;
        minicbor::bytes::encode(self.key.key_bytes(), e, ctx)?;
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
        
        let verifying_key: [u8; 32] = minicbor::bytes::decode(d, &mut ())?;
        let signature = crate::cbor::signature::decode(d, &mut ())?;
        let chain_code: [u8; 32] = minicbor::bytes::decode(d, &mut ())?;
        let attributes: Vec<u8> = minicbor::bytes::decode(d, &mut ())?;

        let key = ExtendedVerifyingKey::new(verifying_key, chain_code)
            .ok_or(minicbor::decode::Error::message("Invalid verifying key curve point"))?;

        Ok(Bootstrap {
            key,
            signature,
            attributes: attributes.into_boxed_slice(),
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
