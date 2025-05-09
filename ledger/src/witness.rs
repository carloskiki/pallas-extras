use bip32::ExtendedVerifyingKey;
use minicbor::{CborLen, Decode, Encode};

use crate::{
    crypto::Signature,
    protocol,
    script::{native, plutus},
};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Set {
    #[n(0)]
    #[cbor(with = "cbor_util::boxed_slice", has_nil)]
    pub verifying_keys: Box<[VerifyingKey]>,
    #[n(1)]
    #[cbor(with = "cbor_util::boxed_slice", has_nil)]
    pub native_scripts: Box<[native::Script]>,
    #[n(2)]
    #[cbor(with = "cbor_util::boxed_slice", has_nil)]
    pub bootstraps: Box<[Bootstrap]>,
    #[n(3)]
    #[cbor(with = "cbor_util::boxed_slice", has_nil)]
    pub plutus_v1: Box<[plutus::Script]>,
    #[n(4)]
    #[cbor(with = "cbor_util::boxed_slice", has_nil)]
    pub plutus_data: Box<[plutus::Data]>,
    #[n(5)]
    #[cbor(with = "cbor_util::boxed_slice", has_nil)]
    pub redeemers: Box<[Redeemer]>,
    #[n(6)]
    #[cbor(with = "cbor_util::boxed_slice", has_nil)]
    pub plutus_v2: Box<[plutus::Script]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct VerifyingKey {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    pub vkey: crate::crypto::VerifyingKey,
    #[n(1)]
    #[cbor(with = "cbor_util::signature")]
    pub signature: Signature,
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
        cbor_util::signature::encode(&self.signature, e, &mut ())?;

        e.bytes(&self.key.chain_code)?.bytes(&self.attributes)?.ok()
    }
}

impl<C> Decode<'_, C> for Bootstrap {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        cbor_util::array_decode(
            4,
            |d| {
                let verifying_key: [u8; 32] = minicbor::bytes::decode(d, &mut ())?;
                let signature = cbor_util::signature::decode(d, &mut ())?;
                let chain_code: [u8; 32] = minicbor::bytes::decode(d, &mut ())?;
                let attributes: Vec<u8> = minicbor::bytes::decode(d, &mut ())?;

                let key = ExtendedVerifyingKey::new(verifying_key, chain_code).ok_or(
                    minicbor::decode::Error::message("Invalid verifying key curve point"),
                )?;

                Ok(Bootstrap {
                    key,
                    signature,
                    attributes: attributes.into_boxed_slice(),
                })
            },
            d,
        )
    }
}

impl<C> CborLen<C> for Bootstrap {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        let attributes: &minicbor::bytes::ByteSlice = (*self.attributes).into();
        let two_arrays_len = 2 * (32.cbor_len(ctx) + 32);

        4.cbor_len(ctx)
            + attributes.cbor_len(ctx)
            + cbor_util::signature::cbor_len(&self.signature, ctx)
            + two_arrays_len
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
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
