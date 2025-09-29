use minicbor_derive::{Encode, Decode, CborLen};

// #[derive(Encode, Decode, CborLen, Clone, Debug)]
// #[cbor(array)]
// enum SampleArrayEncoding<T> {
//     #[n(0)]
//     Unit,
//     #[n(1)]
//     Struct {
//         #[n(0)]
//         field1: String,
//         #[n(1)]
//         field2: bool,
//     },
//     #[n(2)]
//     TupleStruct(#[n(0)] u32, #[n(1)] String),
//     #[n(3)]
//     Generic(#[n(0)] T),
// }

#[derive(Encode, Decode, CborLen, Clone, Debug)]
enum Test {
    #[n(0)]
    Variant
}

fn main() {}
