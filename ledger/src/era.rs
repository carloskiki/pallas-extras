use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(tag_only)]
pub enum Era {
    #[n(0)]
    Byron,
    #[n(1)]
    Shelley,
    #[n(2)]
    Allegra,
    #[n(3)]
    Mary,
    #[n(4)]
    Alonzo,
    #[n(5)]
    Babbage,
    #[n(6)]
    Conway,
}
