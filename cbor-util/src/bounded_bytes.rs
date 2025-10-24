use minicbor::{
    Decoder, Encoder,
    bytes::ByteSlice,
    decode as de, encode as en,
};

pub fn encode<C, W: en::Write>(
    value: &[u8],
    e: &mut Encoder<W>,
    _: &mut C,
) -> Result<(), en::Error<W::Error>> {
    if value.len() <= 64 {
        e.bytes(value)?.ok()
    } else {
        e.begin_bytes()?;
        value
            .chunks(64)
            .try_for_each(|chunk| e.bytes(chunk)?.ok())?;
        e.end()?.ok()
    }
}

pub fn decode<Ctx>(d: &mut Decoder<'_>, _: &mut Ctx) -> Result<Vec<u8>, de::Error> {
    match d.datatype()? {
        minicbor::data::Type::Bytes => {
            let bytes: &ByteSlice = d.decode()?;
            if bytes.len() > 64 {
                Err(de::Error::message("byte slice too long for bounded bytes"))
            } else {
                Ok(bytes.to_vec())
            }
        },
        minicbor::data::Type::BytesIndef => {
            let mut bytes = Vec::with_capacity(64);
            for slice in  d.bytes_iter()? {
                let slice = slice?;
                if slice.len() > 64 {
                    return Err(de::Error::message("byte slice too long for bounded bytes"))
                }
                bytes.extend_from_slice(slice);
            }
            Ok(bytes)
        },
        t => Err(de::Error::type_mismatch(t).at(d.position()))
    }
}

pub fn cbor_len<Ctx>(value: &[u8], ctx: &mut Ctx) -> usize {
    if value.len() <= 64 {
        minicbor::bytes::cbor_len(value, ctx)
    } else {
        2 + value
            .chunks(64)
            .map(|c| minicbor::bytes::cbor_len(c, ctx))
            .sum::<usize>()
    }
}


#[cfg(test)]
mod tests {
    use std::error::Error;

    use minicbor::{Decoder, Encoder};
    use rand::RngCore;

    
    #[test]
    fn roundtrip() -> Result<(), Box<dyn Error>> {
        const LENGTHS: &[usize] = &[0, 1, 20, 40, 63, 64, 65, 128, 200, 256, 257];
        let mut rng = rand::thread_rng();
        let encoder = &mut Encoder::new(Vec::new());
        let ctx = &mut ();
        for len in LENGTHS {
            // Generate Random slice with length `len` using rand
            let mut bytes = vec![0u8; *len];
            rng.fill_bytes(&mut bytes);

            super::encode(&bytes, encoder, ctx)?;
            assert_eq!(encoder.writer().len(), super::cbor_len(&bytes, ctx));
            
            let out = super::decode(&mut Decoder::new(encoder.writer()), ctx)?;
            assert_eq!(out, bytes);
            encoder.writer_mut().clear();
        }
        Ok(())
    }
}
