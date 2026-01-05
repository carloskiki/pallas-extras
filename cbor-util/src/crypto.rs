use tinycbor::{CborLen, Decode, Decoder, Encode, Encoder, Write, collections};

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
