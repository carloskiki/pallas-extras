use tinycbor_derive::{CborLen, Decode, Encode};

use crate::crypto::Blake2b224Digest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub enum Distribution {
    #[n(1)]
    Bootstrap,
    #[n(0)]
    SingleKey(Blake2b224Digest),
}
