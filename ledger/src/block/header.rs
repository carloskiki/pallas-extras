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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Body {
    pub block_number: Number,
    pub slot: slot::Number,
    pub previous_hash: Option<Blake2b256Digest>,
    pub issuer_vkey: crypto::VerifyingKey,
    pub vrf_vkey: crypto::VerifyingKey,
    /// In Babbage and beyond, this serves both as the leader VRF and the nonce VRF.
    pub nonce_vrf: VrfCertificate,
    pub leader_vrf: VrfCertificate,
    pub block_body_size: u32,
    pub block_body_hash: Blake2b256Digest,
    pub operational_certificate: OperationalCertificate,
    pub protocol_version: protocol::Version,
}

const BABBAGE_ARRAY_SIZE: u64 = 10;
const LEGACY_ARRAY_SIZE: u64 = 15;

impl<C> Encode<C> for Body {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let era = self.protocol_version.major.era();

        e.array(if era < protocol::Era::Babbage {
            LEGACY_ARRAY_SIZE
        } else {
            BABBAGE_ARRAY_SIZE
        })?;

        e.u64(self.block_number)?;
        e.u64(self.slot)?;
        if let Some(previous_hash) = self.previous_hash {
            e.bytes(&previous_hash)?;
        } else {
            e.null()?;
        }
        minicbor::bytes::encode(&self.issuer_vkey, e, &mut ())?;
        minicbor::bytes::encode(&self.vrf_vkey, e, &mut ())?;
        e.encode(&self.leader_vrf)?;
        if era < protocol::Era::Babbage {
            e.encode(&self.nonce_vrf)?;
        }
        e.u32(self.block_body_size)?;
        e.bytes(&self.block_body_hash)?;

        if era < protocol::Era::Babbage {
            e.bytes(self.operational_certificate.kes_verifying_key.as_ref())?;
            e.u64(self.operational_certificate.sequence_number)?;
            e.u8(self.operational_certificate.key_period)?;
            e.bytes(&self.operational_certificate.signature.to_bytes())?;
        } else {
            e.encode(&self.operational_certificate)?;
        }

        if era < protocol::Era::Babbage {
            e.encode(self.protocol_version.major)?;
            e.u8(self.protocol_version.minor)?;
        } else {
            e.encode(self.protocol_version)?;
        }
        Ok(())
    }
}

impl<C> CborLen<C> for Body {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        let era = self.protocol_version.major.era();
        let mut size = if era < protocol::Era::Babbage { BABBAGE_ARRAY_SIZE } else { LEGACY_ARRAY_SIZE }.cbor_len(ctx);
        size += self.block_number.cbor_len(ctx);
        size += self.slot.cbor_len(ctx);
        size += minicbor::bytes::CborLenBytes::cbor_len(&self.previous_hash, ctx);
        size += minicbor::bytes::CborLenBytes::cbor_len(&self.issuer_vkey, ctx);
        size += minicbor::bytes::CborLenBytes::cbor_len(&self.vrf_vkey, ctx);
        size += self.leader_vrf.cbor_len(ctx);
        if era < protocol::Era::Babbage {
            size += self.nonce_vrf.cbor_len(ctx);
        }
        size += self.block_body_size.cbor_len(ctx);
        size += minicbor::bytes::CborLenBytes::cbor_len(&self.block_body_hash, ctx);
        if era < protocol::Era::Babbage {
            size += minicbor::bytes::CborLenBytes::cbor_len(
                &self.operational_certificate.kes_verifying_key.as_ref(),
                ctx,
            );
            size += self.operational_certificate.sequence_number.cbor_len(ctx);
            size += self.operational_certificate.key_period.cbor_len(ctx);
            size += cbor_util::signature::cbor_len(&self.operational_certificate.signature, ctx);
        } else {
            size += self.operational_certificate.cbor_len(ctx);
        }

        if era < protocol::Era::Babbage {
            size += self.protocol_version.major.cbor_len(ctx);
            size += self.protocol_version.minor.cbor_len(ctx);
        } else {
            size += self.protocol_version.cbor_len(ctx);
        }
        size
    }
}

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
            let key_period = d.u8()?;
            let signature = cbor_util::signature::decode(d, &mut ())?;
            OperationalCertificate {
                kes_verifying_key,
                sequence_number,
                key_period,
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
    pub key_period: u8,
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
