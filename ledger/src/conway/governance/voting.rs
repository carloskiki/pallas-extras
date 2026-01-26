use crate::conway::governance::action;
use cbor_util::NonEmpty;
use mitsein::vec1::Vec1;
use tinycbor::{
    CborLen, Decode, Encode,
    container::{self, map},
    num::nonzero,
};
use tinycbor_derive::{CborLen, Decode, Encode};

use super::Anchor;

pub mod voter;
pub use voter::Voter;

pub mod threshold;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Procedure<'a> {
    pub vote: Vote,
    pub anchor: Option<Anchor<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub enum Vote {
    #[n(0)]
    No,
    #[n(1)]
    Yes,
    #[n(2)]
    Abstain,
}

pub type Procedures<'a> = Vec1<(Voter<'a>, Vec1<(action::Id<'a>, Procedure<'a>)>)>;

#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub(crate) struct Codec<'a>(pub Procedures<'a>);

impl<'a> From<Codec<'a>> for Procedures<'a> {
    fn from(codec: Codec<'a>) -> Self {
        codec.0
    }
}

impl<'a, 'b> From<&'b Procedures<'a>> for &'b Codec<'a> {
    fn from(asset: &'b Procedures<'a>) -> Self {
        use ref_cast::RefCast;
        Codec::ref_cast(asset)
    }
}

impl Encode for Codec<'_> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        e.map(self.0.len().get())?;
        for (voter, set) in &self.0 {
            voter.encode(e)?;
            <&NonEmpty<_>>::from(set).encode(e)?;
        }
        Ok(())
    }
}

impl CborLen for Codec<'_> {
    fn cbor_len(&self) -> usize {
        let mut len = self.0.len().cbor_len();
        for (policy, bundle) in &self.0 {
            len += policy.cbor_len();
            len += <&NonEmpty<_>>::from(bundle).cbor_len();
        }
        len
    }
}

impl<'a, 'b: 'a> Decode<'b> for Codec<'a> {
    type Error = container::Error<
        nonzero::Error<
            map::Error<
                <Voter<'a> as Decode<'b>>::Error,
                <NonEmpty<Vec<(action::Id<'a>, Procedure<'a>)>> as Decode<'b>>::Error,
            >,
        >,
    >;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        let mut visitor = d.map_visitor()?;
        let mut items = Vec::with_capacity(visitor.remaining().unwrap_or(0));
        while let Some(result) =
            visitor.visit::<Voter<'a>, NonEmpty<Vec<(action::Id<'a>, Procedure<'a>)>>>()
        {
            let (voter, procedures) =
                result.map_err(|e| container::Error::Content(nonzero::Error::Value(e)))?;
            items.push((voter, procedures.0));
        }
        Ok(Codec(items.try_into().map_err(|_| {
            container::Error::Content(nonzero::Error::Zero)
        })?))
    }
}
