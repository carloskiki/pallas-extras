use sha3::{Digest, Sha3_256};
use tinycbor::{Encode, Encoder};
use tinycbor_derive::{CborLen, Decode, Encode};

use crate::{
    byron::address::attributes::Attributes,
    crypto::{Blake2b224, Blake2b224Digest},
};

// TODO: Check which fields are actually used here on mainnet.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Payload<'a> {
    pub root_digest: Blake2b224Digest,
    pub attributes: Attributes<'a>,
    pub address_type: u32, // TODO: This should only be 0 or 2, make sure of that.
}

impl<'a> Payload<'a> {
    pub fn new(
        spending_data: super::Data,
        attributes: super::Attributes<'a>,
        address_type: u32,
    ) -> Self {
        #[derive(Encode)]
        struct Root<'a, 'b> {
            address_type: u32,
            spending_data: super::Data<'a>,
            attributes: super::Attributes<'b>,
        }

        // Arbitrary size that should fit most encodings without resizing
        let mut encoder = Encoder(Vec::with_capacity(64));
        // Unwrap because we know the writer (Vec) can't fail
        let root = Root {
            address_type,
            spending_data,
            attributes,
        };
        root.encode(&mut encoder);

        let root_digest: Blake2b224Digest = Blake2b224::digest(Sha3_256::digest(&encoder.0)).into();
        Payload {
            root_digest,
            attributes: root.attributes,
            address_type,
        }
    }
}
