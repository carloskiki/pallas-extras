use tinycbor_derive::{Encode, Decode, CborLen};

use crate::{conway::script::plutus, crypto::Blake2b256Digest};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub enum Datum<'a> {
    #[n(0)]
    Hash(&'a Blake2b256Digest),
    #[n(1)]
    Data(plutus::Data),
}
