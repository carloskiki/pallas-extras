use minicbor::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Encode, Decode)]
#[cbor(transparent)]
pub struct Script(#[cbor(with = "crate::cbor::boxed_slice")] Box<[u8]>);

// TODO: Implement Encode and Decode. This looks complicated, should try to understand how it works
// to properly represent the different data types.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Data;

impl<C> Encode<C> for Data {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.null()?.ok()
    }
}
impl<C> Decode<'_, C> for Data {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        d.skip().map(|_| Data)
    }
}
