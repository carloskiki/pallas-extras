use blake2::{Blake2b, Blake2b256};
use sha2::{Digest, Sha256, digest::array::Array};
use sha3::{Keccak256, Sha3_256};
use ripemd::{Ripemd160};

pub fn sha2_256(mut data: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&data);
    data.resize(Sha256::output_size(), 0);
    hasher.finalize_into(
        <&mut Array<u8, _>>::try_from(data.as_mut_slice())
            .ok()
            .unwrap(),
    );
    data
}

pub fn sha3_256(mut data: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(&data);
    data.resize(Sha3_256::output_size(), 0);
    hasher.finalize_into(
        <&mut Array<u8, _>>::try_from(data.as_mut_slice())
            .ok()
            .unwrap(),
    );
    data
}

pub fn blake2b256(mut data: Vec<u8>) -> Vec<u8> {
    let mut hasher = Blake2b256::new();
    hasher.update(&data);
    data.resize(Blake2b256::output_size(), 0);
    hasher.finalize_into(
        <&mut Array<u8, _>>::try_from(data.as_mut_slice())
            .ok()
            .unwrap(),
    );
    data
}

pub fn keccak256(mut data: Vec<u8>) -> Vec<u8> {
    let mut hasher = Keccak256::new();
    hasher.update(&data);
    data.resize(Keccak256::output_size(), 0);
    hasher.finalize_into(
        <&mut Array<u8, _>>::try_from(data.as_mut_slice())
            .ok()
            .unwrap(),
    );
    data
}

pub fn blake2b224(mut data: Vec<u8>) -> Vec<u8> {
    let mut hasher = Blake2b::<blake2::digest::consts::U28>::new();
    hasher.update(&data);
    data.resize(Blake2b::<blake2::digest::consts::U28>::output_size(), 0);
    hasher.finalize_into(
        <&mut Array<u8, _>>::try_from(data.as_mut_slice())
            .ok()
            .unwrap(),
    );
    data
}

pub fn ripemd160(mut data: Vec<u8>) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(&data);
    data.resize(Ripemd160::output_size(), 0);
    hasher.finalize_into(
        <&mut Array<u8, _>>::try_from(data.as_mut_slice())
            .ok()
            .unwrap(),
    );
    data
}
