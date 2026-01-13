use digest::generic_array::GenericArray;
use minicbor::{CborLen, Decode, Encode};

use crate::crypto::{self, Blake2b256Digest};

use crate::{protocol, slot};

use super::Number;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Header {
    #[n(0)]
    pub body: Body,
    #[n(1)]
    #[cbor(with = "cbor_util::signature")]
    pub signature: crypto::kes::Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, CborLen)]
pub struct Body {
    #[n(0)]
    pub block_number: Number,
    #[n(1)]
    pub slot: slot::Number,
    #[cbor(n(2), with = "minicbor::bytes")]
    pub previous_hash: Option<Blake2b256Digest>,
    #[cbor(n(3), with = "minicbor::bytes")]
    pub issuer_vkey: crypto::VerifyingKey,
    #[cbor(n(4), with = "minicbor::bytes")]
    pub vrf_vkey: crypto::VerifyingKey,
    #[cbor(skip)]
    pub nonce_vrf: VrfCertificate,
    /// In Babbage and beyond, this serves both as the leader VRF and the nonce VRF.
    #[n(5)]
    pub leader_vrf: VrfCertificate,
    #[n(6)]
    pub block_body_size: u32,
    #[cbor(n(7), with = "minicbor::bytes")]
    pub block_body_hash: Blake2b256Digest,
    #[n(8)]
    pub operational_certificate: OperationalCertificate,
    #[n(9)]
    pub protocol_version: protocol::Version,
}

const BABBAGE_ARRAY_SIZE: u64 = 10;
const LEGACY_ARRAY_SIZE: u64 = 15;

impl<C> Decode<'_, C> for Body {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let array_size = d.array()?;
        if array_size.is_some_and(|size| size != BABBAGE_ARRAY_SIZE  && size != LEGACY_ARRAY_SIZE) {
            return Err(minicbor::decode::Error::message("invalid array size"));
        }
        let block_number = d.u64()?;
        let slot = d.u64()?;
        let previous_hash = d
            .decode::<Option<minicbor::bytes::ByteArray<32>>>()
            .map(|bytes| bytes.map(|b| b.into()))?;

        let issuer_vkey: crypto::VerifyingKey = minicbor::bytes::decode(d, &mut ())?;
        let vrf_vkey: crypto::VerifyingKey = minicbor::bytes::decode(d, &mut ())?;

        let first_vrf: VrfCertificate = d.decode()?;
        let protocol_less_than_babbage = matches!(d.datatype()?, Type::Array | Type::ArrayIndef);
        use minicbor::data::Type;
        let (leader_vrf, nonce_vrf): (VrfCertificate, VrfCertificate) =
            if protocol_less_than_babbage {
                (d.decode()?, first_vrf)
            } else {
                // TODO: get the nonce_vrf from the leader_vrf
                (
                    first_vrf,
                    VrfCertificate {
                        hash: [0; 64],
                        proof: [0; 80],
                    },
                )
            };
        let block_body_size = d.u32()?;
        let block_body_hash: Blake2b256Digest = minicbor::bytes::decode(d, &mut ())?;

        let operational_certificate = if protocol_less_than_babbage {
            let kes_verifying_key_bytes: Blake2b256Digest = minicbor::bytes::decode(d, &mut ())?;
            let kes_verifying_key_bytes: GenericArray<_, _> =
                GenericArray::from(kes_verifying_key_bytes);
            let kes_verifying_key = kes::sum::VerifyingKey::from(kes_verifying_key_bytes);
            let sequence_number = d.u64()?;
            let key_period = d.u64()?;
            let signature = cbor_util::signature::decode(d, &mut ())?;
            OperationalCertificate {
                kes_verifying_key,
                sequence_number,
                kes_start_period: key_period,
                signature,
            }
        } else {
            d.decode()?
        };
        
        let protocol_version: protocol::Version = if protocol_less_than_babbage {
            let major = d.decode()?;
            let minor = d.u8()?;
            protocol::Version { major, minor }
        } else {
            d.decode()?
        };

        Ok(Body {
            block_number,
            slot,
            previous_hash,
            issuer_vkey,
            vrf_vkey,
            leader_vrf,
            nonce_vrf,
            block_body_size,
            block_body_hash,
            operational_certificate,
            protocol_version,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct VrfCertificate {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    pub hash: [u8; 64],
    // TODO: use the correct proof type once implemented in an upstream crate.
    #[n(1)]
    #[cbor(with = "minicbor::bytes")]
    pub proof: [u8; 80],
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct OperationalCertificate {
    #[cbor(n(0), with = "kes_verifying_key")]
    pub kes_verifying_key: crypto::kes::VerifyingKey,
    #[n(1)]
    pub sequence_number: u64,
    #[n(2)]
    pub kes_start_period: u64,
    #[cbor(n(3), with = "cbor_util::signature")]
    pub signature: crypto::Signature,
}

mod kes_verifying_key {
    use minicbor::CborLen;

    pub fn encode<C, W: minicbor::encode::Write>(
        value: &super::crypto::kes::VerifyingKey,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        minicbor::bytes::encode(&value.as_ref(), e, &mut ())?;
        Ok(())
    }

    pub fn decode<Ctx>(
        d: &mut minicbor::Decoder<'_>,
        _: &mut Ctx,
    ) -> Result<super::crypto::kes::VerifyingKey, minicbor::decode::Error> {
        let bytes = cbor_util::bytes_iter_collect(d.bytes_iter()?)?;

        super::crypto::kes::VerifyingKey::try_from(bytes.as_ref())
            .map_err(|_| minicbor::decode::Error::message("invalid kes verifying key"))
    }

    pub fn cbor_len<Ctx>(value: &super::crypto::kes::VerifyingKey, ctx: &mut Ctx) -> usize {
        let len = value.as_ref().len();
        len.cbor_len(ctx) + len
    }
}
