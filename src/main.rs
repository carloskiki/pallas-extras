use minicbor::bytes::ByteArray;

const ARRAY: [u8; 28] = [0; 28];

fn main() {
    let v = minicbor::to_vec(ByteArray::from(ARRAY));
    dbg!(v.unwrap().len());
}
