use minicbor::{Decode, Encode};

pub mod block;
pub mod certificate;
pub mod credential;
pub mod pool;
pub mod protocol;
pub mod transaction;
pub mod witness;
pub mod address;
pub mod plutus;
pub mod cbor;
pub mod crypto;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[cbor(tag(30))]
pub struct RealNumber {
    #[n(0)]
    pub numerator: u64,
    #[n(1)]
    pub denominator: u64,
}
