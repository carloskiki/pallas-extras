pub mod relay;
pub use relay::Relay;

pub mod metadata;
pub use metadata::Metadata;

/// Pool identifier, a.k.a. pool key hash.
pub type Id = crate::crypto::Blake2b224Digest;

pub type DnsName = super::Url;
