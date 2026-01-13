use minicbor::{CborLen, Decode, Encode};

use crate::{protocol::RealNumber, Credential};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DelegateRepresentative {
    Credential(Credential),
    Abstain,
    NoConfidence,
}

impl DelegateRepresentative {
    fn tag(&self) -> u8 {
        match self {
            DelegateRepresentative::Credential(Credential::VerificationKey(_)) => 0,
            DelegateRepresentative::Credential(Credential::Script(_)) => 1,
            DelegateRepresentative::Abstain => 2,
            DelegateRepresentative::NoConfidence => 3,
        }
    }
}

impl<C> Encode<C> for DelegateRepresentative {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.u8(self.tag())?;
        match self {
            DelegateRepresentative::Credential(
                Credential::VerificationKey(h) | Credential::Script(h),
            ) => {
                e.array(2)?.u8(self.tag())?;
                minicbor::bytes::encode(h, e, ctx)?;
            }
            _ => {
                e.array(1)?.u8(self.tag())?;
            }
        }
        Ok(())
    }
}

impl<C> Decode<'_, C> for DelegateRepresentative {
    fn decode(d: &mut minicbor::Decoder<'_>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        if d.array()?.is_some_and(|l| l != 1 && l != 2) {
            return Err(minicbor::decode::Error::message("invalid array length").at(d.position()));
        };
        let tag = d.u8()?;
        Ok(match tag {
            0 => DelegateRepresentative::Credential(Credential::VerificationKey(
                minicbor::bytes::decode(d, ctx)?,
            )),
            1 => DelegateRepresentative::Credential(Credential::Script(minicbor::bytes::decode(
                d, ctx,
            )?)),
            2 => DelegateRepresentative::Abstain,
            3 => DelegateRepresentative::NoConfidence,
            _ => return Err(minicbor::decode::Error::message("invalid tag").at(d.position())),
        })
    }
}

impl<C> CborLen<C> for DelegateRepresentative {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        let tag = self.tag();
        tag.cbor_len(ctx)
            + match self {
                DelegateRepresentative::Credential(credential) => {
                    minicbor::bytes::cbor_len(credential.as_ref(), ctx)
                }
                _ => 0,
            }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct VotingThresholds {
    #[n(0)]
    motion_no_confidence: RealNumber,
    #[n(1)]
    update_committee: RealNumber,
    #[n(2)]
    update_committee_no_confidence: RealNumber,
    #[n(3)]
    update_constitution: RealNumber,
    #[n(4)]
    hard_fork_initiation: RealNumber,
    #[n(5)]
    protocol_parameter_network_update: RealNumber,
    #[n(6)]
    protocol_parameter_economic_update: RealNumber,
    #[n(7)]
    protocol_parameter_technical_update: RealNumber,
    #[n(8)]
    protocol_parameter_security_update: RealNumber,
}
