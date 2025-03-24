pub use minicbor::*;

pub mod bool_as_u8 {
    use minicbor::{Decoder, Encoder, decode as de, encode as en};

    pub fn encode<C, W: en::Write>(
        value: &bool,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        e.u8(*value as u8)?.ok()
    }

    pub fn decode<Ctx>(d: &mut Decoder<'_>, _: &mut Ctx) -> Result<bool, de::Error> {
        d.u8().map(|v| v != 0)
    }
}

pub mod boxed_slice {
    use minicbor::{Decoder, Encoder, decode as de, encode as en};

    #[allow(clippy::borrowed_box)]
    pub fn encode<C, W: en::Write, T: en::Encode<C>>(
        value: &Box<[T]>,
        e: &mut Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        e.array(value.len() as u64)?;
        for v in value.iter() {
            e.encode_with(v, ctx)?;
        }
        Ok(())
    }

    pub fn decode<'a, T: de::Decode<'a, Ctx>, Ctx>(
        d: &mut Decoder<'a>,
        ctx: &mut Ctx,
    ) -> Result<Box<[T]>, de::Error> {
        let v: Vec<T> = d.decode_with(ctx)?;
        Ok(v.into_boxed_slice())
    }

    pub fn nil<T>() -> Option<Box<[T]>> {
        Some(Vec::new().into_boxed_slice())
    }

    #[allow(clippy::borrowed_box)]
    pub fn is_nil<T>(v: &Box<[T]>) -> bool {
        v.is_empty()
    }
}

pub mod list_as_map {
    use minicbor::{Decoder, Encoder, decode as de, encode as en};

    #[allow(clippy::borrowed_box)]
    pub fn encode<C, W: en::Write, T: en::Encode<C>, U: en::Encode<C>>(
        value: &Box<[(T, U)]>,
        e: &mut Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        e.map(value.len() as u64)?;
        for (v, u) in value.iter() {
            e.encode_with(v, ctx)?;
            e.encode_with(u, ctx)?;
        }
        Ok(())
    }

    pub fn decode<'a, T: de::Decode<'a, Ctx>, U: de::Decode<'a, Ctx>, Ctx>(
        d: &mut Decoder<'a>,
        ctx: &mut Ctx,
    ) -> Result<Box<[(T, U)]>, de::Error> {
        d.map_iter_with(ctx)?.collect::<Result<Box<[(T, U)]>, _>>()
    }
}

pub mod bounded_bytes {
    use minicbor::{Decoder, Encoder, bytes::ByteVec, decode as de, encode as en};

    #[allow(clippy::borrowed_box)]
    pub fn encode<C, W: en::Write>(
        value: &Box<[u8]>,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), en::Error<W::Error>> {
        if value.len() < 64 {
            e.bytes(value)?.ok()
        } else {
            e.begin_bytes()?;
            value
                .chunks(64)
                .try_for_each(|chunk| e.bytes(chunk)?.ok())?;
            e.end()?.ok()
        }
    }

    pub fn decode<Ctx>(d: &mut Decoder<'_>, _: &mut Ctx) -> Result<Box<[u8]>, de::Error> {
        // TODO: Here we do not check whether it respects the bounded bytes requirements.
        let v: ByteVec = d.decode()?;
        Ok(Vec::from(v).into_boxed_slice())
    }
}

/// For anything that implements `SignatureEncoding`.
pub mod signature {
    use minicbor::{Decoder, Encoder, decode as de, encode as en};
    use signature::SignatureEncoding;

    pub fn encode<C, S, W: en::Write>(
        value: &S,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), en::Error<W::Error>>
    where
        S: SignatureEncoding + TryInto<S::Repr, Error: core::error::Error + Send + Sync + 'static>,
    {
        let repr: S::Repr = value.clone().try_into().map_err(en::Error::custom)?;
        e.bytes(repr.as_ref())?.ok()
    }

    pub fn decode<S, Ctx>(d: &mut Decoder<'_>, _: &mut Ctx) -> Result<S, de::Error>
    where
        S: SignatureEncoding + for<'a> TryFrom<&'a [u8], Error: core::error::Error + Send + Sync + 'static>,
    {
        let bytes = d.bytes()?;
        S::try_from(bytes).map_err(de::Error::custom)
    }
}
