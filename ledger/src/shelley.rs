pub mod address;
pub use address::Address;

pub mod credential;
pub use credential::Credential;


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

