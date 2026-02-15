use crate::shelley::{self, Credential};
use tinycbor::{container::bounded, *};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DelegateRepresentative<'a> {
    Credential(Credential<'a>),
    Abstain,
    NoConfidence,
}

impl Encode for DelegateRepresentative<'_> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        match self {
            DelegateRepresentative::Credential(Credential::VerificationKey(vkey)) => {
                e.array(2)?;
                0.encode(e)?;
                vkey.encode(e)
            }
            DelegateRepresentative::Credential(Credential::Script(script)) => {
                e.array(2)?;
                1.encode(e)?;
                script.encode(e)
            }
            DelegateRepresentative::Abstain => {
                e.array(1)?;
                2.encode(e)
            }
            DelegateRepresentative::NoConfidence => {
                e.array(1)?;
                3.encode(e)
            }
        }
    }
}

impl CborLen for DelegateRepresentative<'_> {
    fn cbor_len(&self) -> usize {
        match self {
            DelegateRepresentative::Credential(Credential::VerificationKey(vkey)) => {
                1 + 1 + vkey.cbor_len()
            }
            DelegateRepresentative::Credential(Credential::Script(script)) => {
                1 + 1 + script.cbor_len()
            }
            DelegateRepresentative::Abstain => 1 + 1,
            DelegateRepresentative::NoConfidence => 1 + 1,
        }
    }
}

impl<'a, 'b: 'a> Decode<'b> for DelegateRepresentative<'a> {
    type Error = container::Error<bounded::Error<tag::Error<shelley::credential::Error>>>;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        use shelley::credential::Error as CredError;

        let mut visitor = d.array_visitor()?;
        let value = match visitor
            .visit::<u64>()
            .ok_or(bounded::Error::Missing)?
            .map_err(|e| bounded::Error::Content(tag::Error::Malformed(e)))?
        {
            0 => DelegateRepresentative::Credential(Credential::VerificationKey(
                visitor
                    .visit::<&crate::crypto::Blake2b224Digest>()
                    .ok_or(bounded::Error::Missing)?
                    .map_err(|e| {
                        bounded::Error::Content(tag::Error::Content(CredError::VerificationKey(e)))
                    })?,
            )),
            1 => DelegateRepresentative::Credential(Credential::Script(
                visitor
                    .visit::<&crate::crypto::Blake2b224Digest>()
                    .ok_or(bounded::Error::Missing)?
                    .map_err(|e| {
                        bounded::Error::Content(tag::Error::Content(CredError::Script(e)))
                    })?,
            )),
            2 => DelegateRepresentative::Abstain,
            3 => DelegateRepresentative::NoConfidence,
            _ => {
                return Err(bounded::Error::Content(tag::Error::InvalidTag).into());
            }
        };
        if visitor.remaining() != Some(0) {
            return Err(bounded::Error::Surplus.into());
        }
        Ok(value)
    }
}
