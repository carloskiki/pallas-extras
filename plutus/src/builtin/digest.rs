use blake2::Blake2b256;
use sha2::{Digest, Sha256};
use sha3::{Keccak256, Sha3_256};

pub fn sha2_256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn sha3_256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn blake2b256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Blake2b256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn keccak256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn blake2b224(data: &[u8]) -> Vec<u8> {
    let mut hasher = blake2::Blake2b::<blake2::digest::consts::U28>::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}
