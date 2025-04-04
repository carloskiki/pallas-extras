use minicbor::{CborLen, Decode, Encode};

use crate::crypto::Blake2b224Digest;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(flat)]
pub enum Script {
    #[n(0)]
    Vkey(#[cbor(n(0), with = "minicbor::bytes")] Blake2b224Digest),
    #[n(1)]
    All(#[cbor(n(0), with = "cbor_util::boxed_slice")] Box<[Script]>),
    #[n(2)]
    Any(#[cbor(n(0), with = "cbor_util::boxed_slice")] Box<[Script]>),
    #[n(3)]
    NofK(
        #[n(0)] u64,
        #[cbor(n(1), with = "cbor_util::boxed_slice")] Box<[Script]>,
    ),
    #[n(4)]
    InvalidBefore(#[n(0)] u64),
    #[n(5)]
    InvalidHereafter(#[n(0)] u64),
}

