use crate::{alonzo::script, crypto::Blake2b256Digest};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Decode, Encode, CborLen)]
pub enum Datum<'a> {
    #[n(0)]
    Hash(&'a Blake2b256Digest),
    #[n(1)]
    Inline(#[cbor(with = "tinycbor::Encoded<script::Data>")] script::Data),
}
