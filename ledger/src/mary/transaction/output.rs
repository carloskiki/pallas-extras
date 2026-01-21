use tinycbor_derive::{CborLen, Decode, Encode};
use crate::{Address, mary::transaction::value::Value};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Output<'a> {
    #[cbor(decode_with = "crate::address::truncating::Address<'a>")]
    pub address: Address<'a>,
    pub value: Value<'a>,
}
