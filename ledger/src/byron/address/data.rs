use tinycbor_derive::{CborLen, Decode, Encode};

use crate::crypto::VerifyingKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Data<'a> {
    #[n(0)]
    VerifyingKey(#[cbor(with = "cbor_util::ExtendedVerifyingKey<'a>")] &'a bip32::ExtendedVerifyingKey),
    #[n(1)]
    Redeem(VerifyingKey),
}
