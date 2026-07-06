use crate::{
    byron::address::{Type, attributes::Attributes},
    crypto::Blake2b224Digest,
};
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Payload {
    pub root_digest: Blake2b224Digest,
    pub attributes: Attributes,
    pub address_type: Type,
}
