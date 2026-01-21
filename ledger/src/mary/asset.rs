use cbor_util::NonEmpty;
use mitsein::vec1::Vec1;
use tinycbor::{
    CborLen, Decode, Encode,
    container::{self, map},
};

pub mod name;
pub use name::Name;

pub type Asset<'a, T> = Vec<(&'a crate::crypto::Blake2b224Digest, Bundle<'a, T>)>;

pub type Bundle<'a, T> = Vec1<(&'a Name, T)>;

// TODO: figure out if `cbor-util` should be merged in `ledger` or if this should be moved there.
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
        for (policy, bundle) in &self.0 {
            policy.encode(e)?;
            <&NonEmpty<_>>::from(bundle).encode(e)?;
        }
        Ok(())
    }
}

impl<T: CborLen> CborLen for Codec<'_, T> {
    fn cbor_len(&self) -> usize {
        let mut len = self.0.len().cbor_len();
        for (policy, bundle) in &self.0 {
            len += policy.cbor_len();
            len += <&NonEmpty<_>>::from(bundle).cbor_len();
        }
        len
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for Codec<'a, T> {
    type Error = container::Error<
        map::Error<
            <&'a crate::crypto::Blake2b224Digest as Decode<'a>>::Error,
            <NonEmpty<Vec<(&'a Name, T)>> as Decode<'a>>::Error,
        >,
    >;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        let mut visitor = d.map_visitor()?;
        let mut items = Vec::with_capacity(visitor.remaining().unwrap_or(0));
        while let Some(result) =
            visitor.visit::<&'a crate::crypto::Blake2b224Digest, NonEmpty<Vec<(&'a Name, T)>>>()
        {
            let (name, NonEmpty(bundle)) = result.map_err(container::Error::Content)?;
            items.push((name, bundle));
        }

        Ok(Codec(items))
    }
}
