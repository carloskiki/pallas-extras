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

pub mod interval;

pub mod update;
pub use update::Update;

pub mod url;
pub use url::Url;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Network(pub u8);

impl Network {
    pub const MAIN: Self = Network(1);
    pub const TEST: Self = Network(0);

    pub fn main(&self) -> bool {
        self.0 == 1
    }

    pub fn test(&self) -> bool {
        self.0 == 0
    }

    pub fn unknown(&self) -> bool {
        self.0 > 1
    }
}

