use std::convert::Infallible;

use tinycbor::{
    CborLen, Decode, Encode,
    container::{self, bounded},
    tag,
};

use crate::{crypto::Blake2b224Digest, shelley::Credential};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Voter<'a> {
    ConstitutionalCommittee(Credential<'a>),
    DelegateRepresentative(Credential<'a>),
    StakePool {
        verifying_key_hash: &'a Blake2b224Digest,
    },
}

#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum Error {
    /// while decoding constitutional committee credential
    ConstitutionalCommittee(#[source] crate::shelley::credential::Error),
    /// while decoding delegate representative credential
    DelegateRepresentative(#[source] crate::shelley::credential::Error),
    /// while decoding stake pool verifying key hash
    StakePool(#[from] container::Error<bounded::Error<Infallible>>),
}

impl Encode for Voter<'_> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        e.array(2)?;
        let (index, k) = match self {
            Voter::ConstitutionalCommittee(Credential::VerificationKey(k)) => (0, k),
            Voter::ConstitutionalCommittee(Credential::Script(k)) => (1, k),
            Voter::DelegateRepresentative(Credential::VerificationKey(k)) => (2, k),
            Voter::DelegateRepresentative(Credential::Script(k)) => (3, k),
            Voter::StakePool {
                verifying_key_hash: k,
            } => (4, k),
        };
        index.encode(e)?;
        k.encode(e)
    }
}

impl CborLen for Voter<'_> {
    fn cbor_len(&self) -> usize {
        1 + 1 + Blake2b224Digest::default().cbor_len()
    }
}

impl<'a, 'b: 'a> Decode<'b> for Voter<'a> {
    type Error = container::Error<bounded::Error<tag::Error<Error>>>;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        use crate::shelley::credential::Error as CredError;
        macro_rules! wrap {
            ($key:ident, $voter:ident, $cred:ident) => {
                Ok(Voter::$voter(Credential::$cred(
                    $key.map_err(|e| {
                        bounded::Error::Content(tag::Error::Content(Error::$voter(
                            CredError::$cred(e),
                        )))
                    })?
                )))
            }
        }

        let mut visitor = d.array_visitor()?;
        let index = visitor
            .visit::<u64>()
            .ok_or(bounded::Error::Missing)?
            .map_err(|e| bounded::Error::Content(tag::Error::Malformed(e)))?;
        let key = visitor
            .visit::<&'b Blake2b224Digest>()
            .ok_or(bounded::Error::Missing)?;
        match index {
            0 => wrap!(key, ConstitutionalCommittee, VerificationKey),
            1 => wrap!(key, ConstitutionalCommittee, Script),
            2 => wrap!(key, DelegateRepresentative, VerificationKey),
            3 => wrap!(key, DelegateRepresentative, Script),
            4 => Ok(Voter::StakePool {
                verifying_key_hash: key.map_err(|e| {
                    bounded::Error::Content(tag::Error::Content(Error::StakePool(e)))
                })?,
            }),
            _ => Err(bounded::Error::Content(tag::Error::InvalidTag).into()),
        }
    }
}
