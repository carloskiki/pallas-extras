#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
use minicbor_derive::{Encode, Decode, CborLen};
enum Test {
    #[n(0)]
    Variant,
}
impl<__C> ::minicbor::Encode<__C> for Test {
    fn encode<W: ::minicbor::encode::Write>(
        &self,
        e: &mut ::minicbor::encode::Encoder<W>,
        ctx: &mut __C,
    ) -> Result<(), ::minicbor::encode::Error<W::Error>> {
        e.array(2)?;
        match self {
            Self::Variant {} => {
                e.i64(0i64)?;
                e.array(0u64)?;
            }
        }
        Ok(())
    }
}
impl<__C> ::minicbor::CborLen<__C> for Test {
    fn cbor_len(&self, ctx: &mut __C) -> usize {
        2.cbor_len(ctx)
            + match self {
                Self::Variant {} => 0i64.cbor_len(ctx) + 0u64.cbor_len(ctx),
            }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Test {
    #[inline]
    fn clone(&self) -> Test {
        Test::Variant
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Test {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "Variant")
    }
}
fn main() {}
