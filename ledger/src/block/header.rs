use digest::generic_array::GenericArray;
use minicbor::{Decode, Encode};

use crate::crypto::{self, Blake2b256Digest};

use crate::protocol;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct Header {
    #[n(0)]
    body: Body,
    #[n(1)]
    #[cbor(with = "cbor_util::signature")]
    signature: crypto::kes::Signature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Body {
    pub block_number: u64,
    pub slot: u64,
    pub previous_hash: Option<[u8; 32]>,
    pub issuer_vkey: crypto::VerifyingKey,
    pub vrf_vkey: crypto::VerifyingKey,
    /// In Babbage and beyond, this serves both as the leader VRF and the nonce VRF.
    pub leader_vrf: VrfCertificate,
    pub nonce_vrf: VrfCertificate,
    pub block_body_size: u32,
    pub block_body_hash: Blake2b256Digest,
    pub kes_verifying_key: crypto::kes::VerifyingKey,
    pub sequence_number: u64,
    pub key_period: u8,
    pub signature: crypto::Signature,
    pub protocol_version: protocol::Version,
}

impl<C> Encode<C> for Body {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(
            if self.protocol_version.major < protocol::MajorVersion::Vasil {
                15
            } else {
                13
            },
        )?;

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
        if self.protocol_version.major < protocol::MajorVersion::Vasil {
            e.encode(&self.nonce_vrf)?;
        }
        e.u32(self.block_body_size)?;
        e.bytes(&self.block_body_hash)?;
        e.bytes(self.kes_verifying_key.as_ref())?;
        e.u64(self.sequence_number)?;
        e.u8(self.key_period)?;
        e.bytes(&self.signature.to_bytes())?;
        if self.protocol_version.major < protocol::MajorVersion::Vasil {
            e.encode(self.protocol_version.major)?;
            e.u8(self.protocol_version.minor)?;
        } else {
            e.encode(self.protocol_version)?;
        }
        Ok(())
    }
}

impl<C> Decode<'_, C> for Body {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let array_size = d.array()?;
        if array_size.is_some() && array_size != Some(13) && array_size != Some(15) {
            return Err(minicbor::decode::Error::message("invalid array size"));
        }
        let block_number = d.u64()?;
        let slot = d.u64()?;
        let previous_hash = d
            .decode::<minicbor::bytes::ByteArray<32>>()
            .map(|bytes| Some(bytes.into()))
            .or_else(|err| {
                if err.is_type_mismatch() && d.null().is_ok() {
                    Ok(None)
                } else {
                    Err(err)
                }
            })?;
        let issuer_vkey: crypto::VerifyingKey = minicbor::bytes::decode(d, &mut ())?;
        let vrf_vkey: crypto::VerifyingKey = minicbor::bytes::decode(d, &mut ())?;
        let leader_vrf: VrfCertificate = d.decode()?;

        use minicbor::data::Type;
        let protocol_less_than_vasil =
            matches!(d.datatype()?, Type::U8 | Type::U16 | Type::U32 | Type::U64);
        let nonce_vrf: VrfCertificate = if protocol_less_than_vasil {
            // TODO: get the nonce_vrf from the leader_vrf
            VrfCertificate {
                hash: vec![],
                proof: [0; 80],
            }
        } else {
            d.decode()?
        };
        let block_body_size = d.u32()?;
        let block_body_hash: Blake2b256Digest = minicbor::bytes::decode(d, &mut ())?;
        let kes_verifying_key_bytes: Blake2b256Digest = minicbor::bytes::decode(d, &mut ())?;
        let kes_verifying_key_bytes: GenericArray<_, _> =
            GenericArray::from(kes_verifying_key_bytes);
        let kes_verifying_key = kes::sum::VerifyingKey::from(kes_verifying_key_bytes);
        let sequence_number = d.u64()?;
        let key_period = d.u8()?;
        let signature = cbor_util::signature::decode(d, &mut ())?;
        let protocol_version: protocol::Version = if protocol_less_than_vasil {
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
            kes_verifying_key,
            sequence_number,
            key_period,
            signature,
            protocol_version,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct VrfCertificate {
    #[n(0)]
    #[cbor(with = "minicbor::bytes")]
    pub hash: Vec<u8>,
    // TODO: use the correct proof type once implemented in an upstream crate.
    #[n(1)]
    #[cbor(with = "minicbor::bytes")]
    pub proof: [u8; 80],
}
