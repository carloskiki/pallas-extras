use macro_rules_attribute::apply;
use rug::Complete;
use tinycbor::*;

pub mod bounded_bytes;
pub use crate::tinycbor::bounded_bytes::BoundedBytes;

pub mod non_empty;

macro_rules! wrapper {
    ($vis:vis struct $name:ident(pub $inner:ty);) => {
        #[derive(ref_cast::RefCast)]
        #[repr(transparent)]
        $vis struct $name(pub $inner);

        impl From<$name> for $inner {
            fn from(wrapper: $name) -> Self {
                wrapper.0
            }
        }

        impl AsRef<$name> for $inner {
            fn as_ref(&self) -> &$name {
                use ref_cast::RefCast;
                $name::ref_cast(&self)
            }
        }
    };
}
use wrapper;

#[apply(wrapper)]
pub struct BigInt(pub rug::Integer);

impl CborLen for BigInt {
    fn cbor_len(&self) -> usize {
        let negative = self.0.is_negative();
        if self.0.as_limbs().len() < 2 {
            #[allow(clippy::unnecessary_cast)] // On windows, this is a u32.
            let mut bits = self.0.as_limbs().first().copied().unwrap_or_default() as u64;
            if negative {
                bits -= 1;
            }
            num::Int { negative, bits }.cbor_len()
        } else {
            let negative = self.0.is_negative();
            let bytes = (&self.0 + if negative { 1u8 } else { 0 })
                .complete()
                .to_digits(rug::integer::Order::Msf);
            1 + BoundedBytes(bytes).cbor_len()
        }
    }
}

impl Encode for BigInt {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        let negative = self.0.is_negative();
        if self.0.as_limbs().len() < 2 {
            #[allow(clippy::unnecessary_cast)] // On windows, this is a u32.
            let mut bits = self.0.as_limbs().first().copied().unwrap_or_default() as u64;
            if negative {
                bits -= 1;
            }

            num::Int { negative, bits }.encode(e)
        } else if negative {
            tag::Tagged::<BoundedBytes, { tag::IanaTag::NegBignum as u64 }>(BoundedBytes(
                (&self.0 + 1u8)
                    .complete()
                    .to_digits(rug::integer::Order::Msf),
            ))
            .encode(e)
        } else {
            tag::Tagged::<BoundedBytes, { tag::IanaTag::PosBignum as u64 }>(BoundedBytes(
                self.0.to_digits(rug::integer::Order::Msf),
            ))
            .encode(e)
        }
    }
}

impl Decode<'_> for BigInt {
    type Error = Error;

    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        match d.datatype().map_err(|e| Error::Int(e.into()))? {
            Type::Int => {
                let int = num::Int::decode(d).map_err(Error::Int)?;
                let big_int = rug::Integer::from(int.bits);
                let big_int = if int.negative { -big_int - 1 } else { big_int };
                Ok(BigInt(big_int))
            }
            Type::Tag => {
                let pre = *d;
                match tag::Tagged::<bounded_bytes::BoundedBytes, { tag::IanaTag::PosBignum as u64 }>::decode(d) {
                    Ok(tagged) => {
                        let bytes = tagged.0.0;
                        let big_int = rug::Integer::from_digits(&bytes, rug::integer::Order::Msf);
                        return Ok(BigInt(big_int));
                    }
                    Err(tag::Error::InvalidTag) => {},
                    Err(e) => return Err(Error::BigInt(e)),
                }
                *d = pre;
                
                tag::Tagged::<bounded_bytes::BoundedBytes, { tag::IanaTag::NegBignum as u64 }>::decode(d)
                    .map_err(Error::BigInt)
                    .map(|tagged| {
                        let bytes = tagged.0.0;
                        let big_int = rug::Integer::from_digits(&bytes, rug::integer::Order::Msf);
                        BigInt(-big_int - 1)
                    })
            }
            _ => Err(Error::Int(primitive::Error::InvalidHeader(InvalidHeader))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("failed to decode integer: {0}")]
    Int(primitive::Error),
    #[error("failed to decode big integer: {0}")]
    BigInt(tag::Error<bounded_bytes::Error>),
}
