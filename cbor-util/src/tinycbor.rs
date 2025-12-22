use std::fmt::{Display, Formatter};

use tinycbor::{CborLen, Decode, Encode, Write, primitive};

macro_rules! impl_transforms {
    ($ty:ty, $inner:ty) => {
        impl From<$ty> for $inner {
            fn from(value: $ty) -> Self {
                value.0
            }
        }

        impl AsRef<$ty> for $inner {
            fn as_ref(&self) -> &$ty {
                // Safety: $ty is a transparent wrapper around $inner
                unsafe { &*(self as *const $inner as *const $ty) }
            }
        }
    };
}

#[repr(transparent)]
pub struct BoundedBytes(pub Vec<u8>);

impl From<BoundedBytes> for Vec<u8> {
    fn from(bounded: BoundedBytes) -> Self {
        bounded.0
    }
}

impl AsRef<BoundedBytes> for Vec<u8> {
    fn as_ref(&self) -> &BoundedBytes {
        // Safety: BoundedBytes is a transparent wrapper around Vec<u8>
        unsafe { &*(self as *const Vec<u8> as *const BoundedBytes) }
    }
}

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
    fn encode<W: Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    Malformed(primitive::Error),
    Overflow,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Malformed(e) => write!(f, "malformed data: {}", e),
            Error::Overflow => write!(f, "chunk exceeds 64 bytes"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Malformed(e) => Some(e),
            Error::Overflow => None,
        }
    }
}

pub struct BigInt
