use std::mem::MaybeUninit;

use sha3::{Digest, Sha3_256};
use tinycbor::{Encode, Encoder};
use tinycbor_derive::{CborLen, Decode, Encode};

use crate::{
    byron::address::{attributes::Attributes, Type},
    crypto::{Blake2b224, Blake2b224Digest},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Payload<'a> {
    pub root_digest: &'a Blake2b224Digest,
    pub attributes: Attributes<'a>,
    pub address_type: Type,
}

impl<'a> Payload<'a> {
    pub fn new(
        root_digest: &'a mut MaybeUninit<Blake2b224Digest>,
        spending_data: super::Data,
        attributes: super::Attributes<'a>,
        address_type: Type,
    ) -> Self {
        #[derive(Encode)]
        struct Root<'a, 'b> {
            address_type: Type,
            spending_data: super::Data<'a>,
            attributes: Attributes<'b>,
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

        root_digest.write(Blake2b224::digest(Sha3_256::digest(&encoder.0)).into());
        Payload {
            // SAFETY: We wrote to `root_digest`, so it is now initialized.
            root_digest: unsafe { root_digest.assume_init_ref() },
            attributes: root.attributes,
            address_type,
        }
    }
}
