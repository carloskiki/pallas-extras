use tinycbor_derive::{CborLen, Decode, Encode};

/// Byron era address attributes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Attributes {
    /// The derivation path used to derive the address.
    ///
    /// See [cardano-wallet](https://cardano-foundation.github.io/cardano-wallet/design/concepts/address-derivation.html).
    #[cbor(n(1), optional)]
    pub key_derivation_path: Option<Box<[u8]>>,
}
