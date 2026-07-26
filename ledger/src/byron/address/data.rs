use crate::crypto::VerifyingKey;
use tinycbor_derive::{CborLen, Decode, Encode};

/// The address identity data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Data<'a> {
    /// The address is backed by a verifying key.
    #[n(0)]
    VerifyingKey(
        #[cbor(with = "cbor_util::ExtendedVerifyingKey<'a>")] &'a bip32::ExtendedVerifyingKey,
    ),
    /// The address is backed by a redeeming key.
    #[n(1)]
    Redeem(#[cbor(with = "cbor_util::VerifyingKey<'a>")] &'a VerifyingKey),
}
