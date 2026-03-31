use tinycbor_derive::{CborLen, Decode, Encode};

pub mod confirm;
pub use confirm::Confirm;

pub mod propose;
pub use propose::Propose;

#[derive(Debug, Encode, Decode, CborLen)]
#[cbor(
    naked,
    decode_bound = "D: tinycbor::Decode<'_>",
    encode_bound = "D: tinycbor::Encode",
    len_bound = "D: tinycbor::CborLen"
)]
pub struct VersionTable<D> {
    pub versions: Vec<(Version, D)>,
}

pub type Version = u16;
