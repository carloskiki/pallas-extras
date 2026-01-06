pub mod array_option;
pub use array_option::ArrayOption;

pub mod big_int;
pub use big_int::BigInt;

pub mod bounded_bytes;
pub use bounded_bytes::BoundedBytes;

pub mod crypto;
pub use crypto::Signature;

pub mod mitsein;
pub use mitsein::NonEmpty;

pub mod bytes;
pub use bytes::Bytes;

// TODO: remove once useless
pub mod inspect;
pub use inspect::{Inspect, Inspector};

pub type ExtendedVerifyingKey<'a> = Bytes<'a, bip32::ExtendedVerifyingKey>;

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
