pub mod header;

pub use header::Header;
use minicbor::{CborLen, Decode, Encode};

use crate::{transaction, witness};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block {
    #[n(0)]
    pub header: Header,
    #[cbor(n(1), with = "cbor_util::boxed_slice")]
    pub transaction_bodies: Box<[transaction::Body]>,
    #[cbor(n(2), with = "cbor_util::boxed_slice")]
    pub witness_sets: Box<[witness::Set]>,
    #[cbor(n(3), with = "cbor_util::list_as_map")]
    pub auxiliary_data: Box<[(u16, transaction::Data)]>,
    #[cbor(
        n(4),
        decode_with = "cbor_util::boxed_slice::decode",
        nil = "cbor_util::boxed_slice::nil",
        encode_with = "cbor_util::boxed_slice::encode"
    )]
    pub invalid_transactions: Box<[u16]>,
}

pub type Number = u64;
