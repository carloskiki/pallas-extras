use tinycbor_derive::{CborLen, Decode, Encode};

pub mod body;
pub use body::Body;

pub mod input;
pub use input::Input;

pub mod metadatum;
pub use metadatum::Metadatum;

pub mod output;
pub use output::Output;

pub mod witness;

pub type Id = crate::crypto::Blake2b256Digest;
pub type Index = u16;
pub type Coin = u64;
pub type Data<'a> = Vec<(metadatum::Label, Metadatum<'a>)>;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Transaction<'a> {
    pub body: Body<'a>,
    pub witnesses: witness::Set<'a>,
    pub metadata: Option<Data<'a>>,
}
