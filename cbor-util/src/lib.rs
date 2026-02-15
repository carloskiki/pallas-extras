// TODO: figure out if `cbor-util` should be merged in `ledger` or if this should be moved there.
pub mod bytes;
pub use bytes::Bytes;

pub mod big_int;
pub use big_int::BigInt;

pub mod bounded_bytes;
pub use bounded_bytes::BoundedBytes;

// TODO: remove if useless
pub mod crypto;

// TODO: remove once useless
pub mod inspect;
pub use inspect::{Inspect, Inspector};

pub mod mitsein;
pub use mitsein::NonEmpty;

pub mod net;
pub use net::{Ipv4Addr, Ipv6Addr};

pub mod option;
pub use option::Array;

pub mod set;
pub use set::Set;

pub type ExtendedVerifyingKey<'a> = Bytes<'a, bip32::ExtendedVerifyingKey>;
pub type VerifyingKey<'a> = Bytes<'a, ed25519_dalek::pkcs8::PublicKeyBytes>;
pub type Signature<'a> = Bytes<'a, ed25519_dalek::Signature>;

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

        impl<'a> From<&'a $inner> for &'a $name {
            fn from(value: &'a $inner) -> Self {
                use ref_cast::RefCast;
                $name::ref_cast(value)
            }
        }

        impl<'a> From<&'a $name> for &'a $inner {
            fn from(value: &'a $name) -> Self {
                &value.0
            }
        }
    };
}
use wrapper;

#[macro_export]
macro_rules! sparse_struct_impl {
    ($type:ty) => {
        const _: () = {
            use tinycbor::{
                CborLen, Decode, Encode, Encoder, Write,
                container::{self, bounded, map},
                primitive,
            };
            impl CborLen for $type {
                fn cbor_len(&self) -> usize {
                    let params = self.as_ref();
                    params.len().cbor_len()
                        + params.iter().map(|param| param.cbor_len()).sum::<usize>()
                }
            }

            impl Encode for $type {
                fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
                    let params = self.as_ref();
                    e.map(params.len())?;
                    params.iter().try_for_each(|param| param.encode(e))
                }
            }

            impl Decode<'_> for $type {
                type Error = container::Error<bounded::Error<map::Error<primitive::Error, Error>>>;

                fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
                    let mut params = Self::default();
                    let mut decode_param = |d: &mut tinycbor::Decoder<'_>| {
                        let param = Decode::decode(d).map_err(|e| {
                            container::Error::Content(match e {
                                tinycbor::tag::Error::Malformed(error) => {
                                    bounded::Error::Content(map::Error::Key(error))
                                }
                                tinycbor::tag::Error::InvalidTag => bounded::Error::Surplus,
                                tinycbor::tag::Error::Content(inner) => {
                                    bounded::Error::Content(map::Error::Value(inner))
                                }
                            })
                        })?;
                        if !params.insert(param) {
                            return Err(container::Error::Content(bounded::Error::Surplus));
                        }
                        Ok(())
                    };

                    if let Some(len) = d.map_visitor()?.remaining() {
                        for _ in 0..len {
                            decode_param(d)?;
                        }
                    } else {
                        while d.datatype()? != tinycbor::Type::Break {
                            decode_param(d)?;
                        }
                        d.next().expect("found break").expect("valid break");
                    };
                    Ok(params)
                }
            }
        };
    };
}
