use crate::crypto;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Operational<'a> {
    /// KES "hot" verifying key
    #[cbor(with = "cbor_util::Bytes<'a, crypto::kes::VerifyingKey>")]
    pub signer: &'a crypto::kes::VerifyingKey,
    /// KES sequence number
    pub sequence_number: u32,
    /// KES period
    pub period: u32,
    /// signature for certificate
    #[cbor(with = "cbor_util::Signature<'a>")]
    pub signature: &'a crypto::Signature,
}
