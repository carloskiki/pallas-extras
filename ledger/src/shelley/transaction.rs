pub mod body;
pub use body::Body;

pub mod input;
pub use input::Input;

pub mod output;
pub use output::Output;

pub type Id = crate::crypto::Blake2b256Digest;
pub type Coin = u64;

pub struct Transaction {
    
}
