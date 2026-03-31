use bytes::Bytes;
use tinycbor::Decode;

/// `T` encoded as cbor bytes.
///
/// The value `T` can be accessed by decoding the bytes with [`Self::decode`]. This is needed
/// because the ledger types borrow from their encoding instead of copying data.
pub struct Encoded<T> {
    pub bytes: Bytes,
    pub(crate) _phantom: std::marker::PhantomData<T>,
}

impl<T> Encoded<T> {
    /// Create a new `Encoded` value from the given bytes.
    pub(crate) fn new(bytes: Bytes) -> Self {
        Self {
            bytes,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Access the value by decoding it.
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

/// Errors that can occur while decoding an encoded value.
#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum Error<E> {
    /// error while decoding value
    Value(#[from] E),
    /// encoded value contains trailing content
    Trailing,
}
