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
