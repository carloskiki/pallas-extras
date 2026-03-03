use cbor_util::NonEmpty;
use mitsein::vec1::Vec1;
use tinycbor::{
    CborLen, Decode, Encode,
    container::{self, map},
};

pub mod name;
pub use name::Name;

use crate::Unique;

pub type Asset<'a, T> = Unique<Vec<(&'a crate::crypto::Blake2b224Digest, Bundle<'a, T>)>, false>;

pub type Bundle<'a, T> = Unique<Vec1<(&'a Name, T)>, false>;

#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub(crate) struct Codec<'a, T>(Asset<'a, T>);

impl<'a, T> From<Codec<'a, T>> for Asset<'a, T> {
    fn from(codec: Codec<'a, T>) -> Self {
        codec.0
    }
}

impl<'a, 'b, T> From<&'b Asset<'a, T>> for &'b Codec<'a, T> {
    fn from(asset: &'b Asset<'a, T>) -> Self {
        use ref_cast::RefCast;
        Codec::ref_cast(asset)
    }
}

impl<T: Encode> Encode for Codec<'_, T> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        e.map(self.0.len())?;
        for (policy, bundle) in self.0.iter() {
            policy.encode(e)?;
            <&NonEmpty<_>>::from(&**bundle).encode(e)?;
        }
        Ok(())
    }
}

impl<T: CborLen> CborLen for Codec<'_, T> {
    fn cbor_len(&self) -> usize {
        let mut len = self.0.len().cbor_len();
        for (policy, bundle) in self.0.iter() {
            len += policy.cbor_len();
            len += <&NonEmpty<_>>::from(&**bundle).cbor_len();
        }
        len
    }
}

// In pre-conway eras, bundles that are empty are pruned from the asset list.
// https://github.com/IntersectMBO/cardano-ledger/pull/5145#discussion_r2186204681
impl<'a, T: Decode<'a>> Decode<'a> for Codec<'a, T> {
    type Error = container::Error<
        map::Error<
            <&'a crate::crypto::Blake2b224Digest as Decode<'a>>::Error,
            <Vec<(&'a Name, T)> as Decode<'a>>::Error,
        >,
    >;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        // TODO: Check whether duplicates are merged or first one is kept.
        // Special handling would be required for merge.
        let mut visitor = d.map_visitor()?;
        let size_hint = visitor.remaining();
        crate::unique::decode_dedup_by_key(
            || loop {
                match visitor.visit::<&'a crate::crypto::Blake2b224Digest, Unique<Vec<(&'a Name, T)>, false>>()? {
                    Ok((policy, bundle)) => {
                        let Ok(bundle) = Vec1::try_from(bundle.0) else {
                            continue;
                        };
                        return Some(Ok((policy, Unique(bundle))))
                    },
                    Err(e) => return Some(Err(container::Error::Content(e))),
                };
            },
            |(k, _)| k,
            size_hint,
        ).map(|(_, unique)| Self(unique))
    }
}
