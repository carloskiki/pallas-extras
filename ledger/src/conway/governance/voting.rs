use minicbor::{CborLen, Decode, Encode};

use crate::{Credential, crypto::Blake2b224Digest};

use super::{action, Anchor};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Procedure {
    #[n(0)]
    pub vote: Vote,
    #[n(1)]
    pub anchor: Option<Anchor>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(transparent)]
pub struct Set(#[cbor(with = "cbor_util::list_as_map")] pub Box<[(action::Id, Procedure)]>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(index_only)]
pub enum Vote {
    #[n(0)]
    No,
    #[n(1)]
    Yes,
    #[n(2)]
    Abstain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Voter {
    ConstitutionalCommittee(Credential),
    DelegateRepresentative(Credential),
    StakePool {
        verifying_key_hash: Blake2b224Digest,
    },
}

impl Voter {
    fn tag(&self) -> u8 {
        match self {
            Voter::ConstitutionalCommittee(Credential::VerificationKey(_)) => 0,
            Voter::ConstitutionalCommittee(Credential::Script(_)) => 1,
            Voter::DelegateRepresentative(Credential::VerificationKey(_)) => 2,
            Voter::DelegateRepresentative(Credential::Script(_)) => 3,
            Voter::StakePool { .. } => 4,
        }
    }
    fn bytes(&self) -> &Blake2b224Digest {
        match self {
            Voter::ConstitutionalCommittee(cred) | Voter::DelegateRepresentative(cred) => {
                cred.as_ref()
            }
            Voter::StakePool {
                verifying_key_hash: h,
            } => h,
        }
    }
}

impl<C> Encode<C> for Voter {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(2)?.u8(self.tag())?.bytes(self.bytes())?.ok()
    }
}

impl<C> Decode<'_, C> for Voter {
    fn decode(d: &mut minicbor::Decoder<'_>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        cbor_util::array_decode(
            2,
            |d| {
                let tag = d.u8()?;
                let hash: Blake2b224Digest = minicbor::bytes::decode(d, ctx)?;
                Ok(match tag {
                    0 => Voter::ConstitutionalCommittee(Credential::VerificationKey(hash)),
                    1 => Voter::ConstitutionalCommittee(Credential::Script(hash)),
                    2 => Voter::DelegateRepresentative(Credential::VerificationKey(hash)),
                    3 => Voter::DelegateRepresentative(Credential::Script(hash)),
                    4 => Voter::StakePool {
                        verifying_key_hash: hash,
                    },
                    _ => {
                        return Err(
                            minicbor::decode::Error::message("unknown voter tag").at(d.position())
                        );
                    }
                })
            },
            d,
        )
    }
}
impl<C> CborLen<C> for Voter {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        2.cbor_len(ctx) + self.tag().cbor_len(ctx) + minicbor::bytes::cbor_len(self.bytes(), ctx)
    }
}
