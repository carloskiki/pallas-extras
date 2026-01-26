pub type Models = Vec<(u8, Vec<i64>)>;

pub(crate) mod model {
    use tinycbor::{
        CborLen, Decode, Encode, container::{self, map}, num
    };

    #[derive(ref_cast::RefCast)]
    #[repr(transparent)]
    pub struct Codec(pub super::Models);

    impl From<Codec> for super::Models {
        fn from(codec: Codec) -> Self {
            codec.0
        }
    }

    impl<'a> From<&'a super::Models> for &'a Codec {
        fn from(models: &'a super::Models) -> Self {
            use ref_cast::RefCast;
            Codec::ref_cast(models)
        }
    }

    impl<'a> Decode<'a> for Codec {
        type Error = container::Error<map::Error<num::Error, <Vec<i64> as Decode<'a>>::Error>>;

        fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
            let mut visitor = d.map_visitor()?;
            let mut items = Vec::with_capacity(visitor.remaining().unwrap_or(0));
            while let Some(result) = visitor.visit::<num::U8, Vec<i64>>() {
                let (key, value) = result.map_err(container::Error::Content)?;
                items.push((key.0, value));
            }
            Ok(Codec(items))
        }
    }

    impl Encode for Codec {
        fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
            e.map(self.0.len())?;
            for (k, v) in &self.0 {
                num::U8(*k).encode(e)?;
                v.encode(e)?;
            }
            Ok(())
        }
    }

    impl CborLen for Codec {
        fn cbor_len(&self) -> usize {
            let mut len = self.0.len().cbor_len();
            for (k, v) in &self.0 {
                len += num::U8(*k).cbor_len();
                len += v.cbor_len();
            }
            len
        }
    }
}
