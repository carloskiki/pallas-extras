use displaydoc::Display;
use thiserror::Error;
use tinycbor::Decode;

/// `T` encoded as cbor bytes.
///
/// It is bad practice to decode and then re-encode values when their hashes are computed based
/// on their encoding. Peers might encode values in a slightly different way that is
/// still considered valid, yielding different hashes for the same data.
pub struct Encoded<T> {
    pub bytes: Box<[u8]>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Encoded<T> {
    // Access the value by decoding it.
    pub fn decode<'a>(&'a self) -> Result<T, Error<T::Error>>
    where
        T: Decode<'a>,
    {
        let mut d = tinycbor::Decoder(&self.bytes);
        let value = T::decode(d)?;
        if d.0.len() != 0 {
            return Err(Error::Trailing);
        }
    }
}

#[derive(Debug, Display, Error)]
pub enum Error<E> {
    /// Error while decoding vlaue.
    Value(#[from] E),
    /// Encoded value contains trailing content.
    Trailing,
}
