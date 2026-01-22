use tinycbor_derive::{CborLen, Decode, Encode};

pub mod address;
pub use address::Address;

pub mod block;
pub use block::Block;

pub mod certificate;
pub use certificate::Certificate;

pub mod credential;
pub use credential::Credential;

pub mod pool;

pub mod protocol;

pub mod script;
pub use script::Script;

pub mod transaction;
pub use transaction::Transaction;

pub mod update;
pub use update::Update;

pub mod url;
pub use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub enum Network {
    #[n(0)]
    Test = 0,
    #[n(1)]
    Main = 1,
}
