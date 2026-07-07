use crate::{
    byron::address::{Type, attributes::Attributes},
    crypto::Blake2b224Digest,
};
use tinycbor_derive::{CborLen, Decode, Encode};

/// Byron Era address payload.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Payload {
    /// The root digest of the address.
    ///
    /// This commits the address [`Type`], the spending [`Data`](super::Data), and the
    /// [`Attributes`] of the address in a single hash.
    ///
    /// A new root_digest can be derived using the [`root_digest`](super::root_digest) function.
    pub root_digest: Blake2b224Digest,
    /// Extra attributes of the address.
    pub attributes: Attributes,
    /// The type of the address.
    pub address_type: Type,
}
