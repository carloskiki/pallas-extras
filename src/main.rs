use blake2::Blake2b;
use digest::consts::U28;
use ponk::shelley::ChainPointer;
use sha2::Digest;

fn main() {
    let pointer = ChainPointer {
        slot: 45,
        tx_index: 254,
        cert_index: 34,
    };

    let pointer_bytes: Vec<u8> = pointer.into_iter().collect();
    println!("Pointer: {:?}", pointer_bytes);
}
