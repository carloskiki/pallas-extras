use bytes::Bytes;
use tinycbor::Decode;

use crate::typefu::map::TypeMap;

/// `T` encoded as cbor bytes.
///
/// It is bad practice to decode and then re-encode values when their hashes are computed based
/// on their encoding. Peers might encode values in a slightly different way that is
/// still considered valid, yielding different hashes for the same data.
pub struct Encoded<T> {
    pub bytes: Bytes,
    pub(crate) _phantom: std::marker::PhantomData<T>,
}

impl<T> Encoded<T> {
    // Access the value by decoding it.
    pub fn decode<'a>(&'a self) -> Result<T, Error<T::Error>>
    where
        T: Decode<'a>,
    {
        let mut d = tinycbor::Decoder(&self.bytes);
        let value = T::decode(&mut d)?;
        if d.0.len() != 0 {
            return Err(Error::Trailing);
        }
        Ok(value)
    }
}

#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum Error<E> {
    /// error while decoding value
    Value(#[from] E),
    /// encoded value contains trailing content
    Trailing,
}
