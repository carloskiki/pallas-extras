use thiserror::Error;
use tinycbor::*;
use macro_rules_attribute::apply;

#[apply(super::wrapper)]
pub struct BoundedBytes(pub Vec<u8>);

impl CborLen for BoundedBytes {
    fn cbor_len(&self) -> usize {
        if self.0.len() <= 64 {
            self.0.cbor_len()
        } else {
            2 + self.0.chunks(64).map(|c| c.cbor_len()).sum::<usize>()
        }
    }
}

impl Encode for BoundedBytes {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        if self.0.len() <= 64 {
            self.0.encode(e)
        } else {
            e.begin_bytes()?;
            self.0.chunks(64).try_for_each(|chunk| chunk.encode(e))?;
            e.end()
        }
    }
}

impl Decode<'_> for BoundedBytes {
    type Error = Error;

    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        d.bytes_iter()
            .map_err(Error::Malformed)?
            .try_fold(Vec::with_capacity(64), |mut bytes, chunk| {
                let chunk = chunk.map_err(Error::Malformed)?;
                if chunk.len() > 64 {
                    return Err(Error::Overflow);
                }
                bytes.extend_from_slice(chunk);
                Ok(bytes)
            })
            .map(BoundedBytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
pub enum Error {
    #[error("malformed bytes: {0}")]
    Malformed(#[from] primitive::Error),
    #[error("chunk exceeds 64 bytes")]
    Overflow,
}

