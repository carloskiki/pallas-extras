pub mod header;
pub use header::Header;

use crate::crypto::Blake2b256Digest;

pub type Number = u64;
pub type Size = u32;
pub type Id = Blake2b256Digest;

pub struct Block {}
