use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Models {
    #[cbor(n(0), optional, decode_with = "Box<[i64; 166]>")]
    plutus_v1: Option<Box<[i64; 166]>>,
    #[cbor(n(1), optional, decode_with = "Box<[i64; 175]>")]
    plutus_v2: Option<Box<[i64; 175]>>,
}
