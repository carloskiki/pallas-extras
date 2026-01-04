use std::convert::Infallible;

use macro_rules_attribute::apply;
use tinycbor::{
    CborLen, Decode, Decoder, Encode, Encoder, Write,
    collections::{self, fixed},
};
use zerocopy::{transmute, transmute_ref};

#[apply(super::wrapper)]
pub struct ExtendedVerifyingKey(pub bip32::ExtendedVerifyingKey);

impl Encode for ExtendedVerifyingKey {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        let bytes: &[u8; 64] = transmute_ref!(&self.0);
        bytes.encode(e)
    }
}

impl<'a, 'b> Decode<'b> for &'a ExtendedVerifyingKey
where
    'b: 'a,
{
    type Error = fixed::Error<Infallible>;

    fn decode(d: &mut Decoder<'_>) -> Result<Self, Self::Error> {
        let bytes: &[u8; 64] = Decode::decode(d)?;
        let [key, chain_code]: [[u8; 32]; 2] = transmute!(bytes);
        Ok(ExtendedVerifyingKey::ref_cast(&bip32::ExtendedVerifyingKey { key, chain_code }))
    }
}

impl Decode<'_> for ExtendedVerifyingKey {
    type Error = fixed::Error<Infallible>;

    fn decode(d: &mut Decoder<'_>) -> Result<Self, Self::Error> {
        let dbg_decoder = *d;
        let bytes: [u8; 64] = Decode::decode(d).map_err(|e: fixed::Error<_>| {
            let bytes: &[u8] = Decode::decode(&mut Decoder(dbg_decoder.0)).unwrap();
            dbg!("decoded this many bytes: ", bytes.len());
            
            e.map(|e| match e {})
        })?;
        let [key, chain_code]: [[u8; 32]; 2] = transmute!(bytes);
        Ok(Self(bip32::ExtendedVerifyingKey { key, chain_code }))
    }
}

impl CborLen for ExtendedVerifyingKey {
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
