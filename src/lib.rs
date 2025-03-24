use digest::consts::{U28, U32};

pub mod byron;
pub mod network;

pub type Blake2b224 = blake2::Blake2b<U28>;
type Blake2b224Digest = [u8; 28];

pub type Blake2b256 = blake2::Blake2b<U32>;
type Blake2b256Digest = [u8; 32];
