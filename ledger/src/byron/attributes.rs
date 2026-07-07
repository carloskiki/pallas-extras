use tinycbor_derive::{CborLen, Decode, Encode};

/// Generic attributes.
///
/// In the Byron era, attributes were liberally attached to many parts of the ledger, but remained
/// unused. This ensures that places where attributes are expected contain the "empty" attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Attributes;
