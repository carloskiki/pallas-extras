use std::convert::Infallible;

use tinycbor::{
    CborLen, Decode, Decoder, Encode, Encoder, Write,
    collections::{self, fixed},
};
use zerocopy::transmute_ref;

#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct ExtendedVerifyingKey<'a>(pub &'a bip32::ExtendedVerifyingKey);

impl<'a> From<ExtendedVerifyingKey<'a>> for &'a bip32::ExtendedVerifyingKey {
    fn from(wrapper: ExtendedVerifyingKey<'a>) -> Self {
        wrapper.0
    }
}

impl<'a, 'b> From<&'b &'a bip32::ExtendedVerifyingKey> for &'b ExtendedVerifyingKey<'a> {
    fn from(value: &'b &'a bip32::ExtendedVerifyingKey) -> Self {
        use ref_cast::RefCast;
        ExtendedVerifyingKey::ref_cast(value)
    }
}

impl Encode for ExtendedVerifyingKey<'_> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        let bytes: &[u8; 64] = transmute_ref!(self.0);
        bytes.encode(e)
    }
}

impl<'a, 'b> Decode<'b> for ExtendedVerifyingKey<'a>
where
    'b: 'a,
{
    type Error = fixed::Error<Infallible>;

    fn decode(d: &mut Decoder<'b>) -> Result<Self, Self::Error> {
        let bytes: &[u8; 64] = Decode::decode(d)?;
        Ok(ExtendedVerifyingKey(transmute_ref!(bytes)))
    }
}

impl<'a> CborLen for ExtendedVerifyingKey<'a> {
    fn cbor_len(&self) -> usize {
        64.cbor_len() + 64
    }
}

#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct Signature<S>(pub S);

impl<S> Signature<S> {
    pub fn into(self) -> S {
        self.0
    }
}

impl<'a, S> From<&'a S> for &'a Signature<S> {
    fn from(value: &'a S) -> Self {
        use ref_cast::RefCast;
        Signature::ref_cast(value)
    }
}

impl<'a, S> Decode<'a> for Signature<S>
where
    S: signature::SignatureEncoding,
    <S as TryFrom<&'a [u8]>>::Error: core::error::Error + 'static,
{
    type Error = collections::Error<<S as TryFrom<&'a [u8]>>::Error>;

    fn decode(d: &mut Decoder<'a>) -> Result<Self, Self::Error> {
        let bytes = Decode::decode(d)?;
        S::try_from(bytes)
            .map_err(collections::Error::Element)
            .map(Signature)
    }
}

impl<S> Encode for Signature<S>
where
    S: signature::SignatureEncoding,
{
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        let repr = self.0.clone().to_bytes();
        repr.as_ref().encode(e)
    }
}

impl<S> CborLen for Signature<S>
where
    S: signature::SignatureEncoding,
{
    fn cbor_len(&self) -> usize {
        let len = self.0.encoded_len();
        len.cbor_len() + len
    }
}
