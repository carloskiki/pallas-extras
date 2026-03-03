use cbor_util::NonEmpty;
use mitsein::vec1::Vec1;
use tinycbor::{
    CborLen, Decode, Encode,
    container::{self, map},
    num::nonzero,
};

use crate::{
    Unique,
    mary::asset::{Bundle, Name},
};

pub type Asset<'a, T> = Unique<Vec1<(&'a crate::crypto::Blake2b224Digest, Bundle<'a, T>)>, false>;

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
        e.map(self.0.len().get())?;
        for (policy, bundle) in &**self.0 {
            policy.encode(e)?;
            <&NonEmpty<_>>::from(&**bundle).encode(e)?;
        }
        Ok(())
    }
}

impl<T: CborLen> CborLen for Codec<'_, T> {
    fn cbor_len(&self) -> usize {
        let mut len = self.0.len().cbor_len();
        for (policy, bundle) in &**self.0 {
            len += policy.cbor_len();
            len += <&NonEmpty<_>>::from(&**bundle).cbor_len();
        }
        len
    }
}

impl<'a, 'b: 'a, T: Decode<'b>> Decode<'b> for Codec<'a, T> {
    type Error = container::Error<
        nonzero::Error<
            map::Error<
                <&'a crate::crypto::Blake2b224Digest as Decode<'b>>::Error,
                <Unique<Vec1<(&'a Name, T)>, false> as Decode<'b>>::Error,
            >,
        >,
    >;

    fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
        // TODO: Check whether duplicates are merged or first one is kept.
        // Special handling would be required for merge.
        let mut visitor = d.map_visitor()?;
        let size_hint = visitor.remaining();
        crate::unique::decode_dedup_by_key(
            || visitor.visit::<&'a crate::crypto::Blake2b224Digest, Unique<Vec1<(&'a Name, T)>, false>>(),
            |(k, _)| k,
            size_hint,
        )
        .map_err(|e| container::Error::Content(nonzero::Error::Value(e)))
        .and_then(|(_, Unique::<_, false>(content))| {
            let Ok(non_empty) = Vec1::try_from(content) else {
                return Err(container::Error::Content(nonzero::Error::Zero));
            };
            Ok(Codec(Unique(non_empty)))
        })
    }
}
