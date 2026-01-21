use displaydoc::Display;
use thiserror::Error;
use tinycbor::{CborLen, Decode, Decoder, Encode, Encoder, Write, container};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Address<'a> {
    Shelley(crate::shelley::Address<'a>),
    Byron(crate::byron::Address<'a>),
}

#[derive(Debug, Error, Display)]
/// An error occurred while decoding an address.
pub enum Error {
    /// while decoding a Shelley era address
    Shelley(#[from] <crate::shelley::Address<'static> as TryFrom<&'static [u8]>>::Error),
    /// while decoding a Byron era address
    Byron(#[from] <crate::byron::Address<'static> as Decode<'static>>::Error),
}

impl Encode for Address<'_> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        match self {
            Address::Shelley(address) => address.encode(e),
            Address::Byron(address) => address.encode(e),
        }
    }
}

impl<'a, 'b: 'a> Decode<'b> for Address<'a> {
    type Error = container::Error<Error>;

    fn decode(d: &mut Decoder<'b>) -> Result<Self, Self::Error> {
        let bytes: &'b [u8] = Decode::decode(d)?;
        if let Some(first) = bytes.first()
            && (first >> 4) == 0b1000
        {
            Decode::decode(&mut Decoder(bytes))
                .map_err(|e| container::Error::Content(Error::Byron(e)))
                .map(Address::Byron)
        } else {
            crate::shelley::Address::try_from(bytes)
                .map_err(|e| container::Error::Content(Error::Shelley(e)))
                .map(Address::Shelley)
        }
    }
}

impl CborLen for Address<'_> {
    fn cbor_len(&self) -> usize {
        match self {
            Address::Shelley(address) => address.cbor_len(),
            Address::Byron(address) => {
                let len = address.cbor_len();
                len.cbor_len() + len
            }
        }
    }
}

/// Truncting address decoder.
///
/// In `mary` and `alonzo` eras, a bug in the ledger implementation allowed for longer than
/// expected shelley addresses to be accepted by the node. These addresses were decoded by
/// truncating the extra bytes.
pub(crate) mod truncating {
    use tinycbor::{Decode, Decoder, container};

    pub struct Address<'a>(pub super::Address<'a>);

    impl<'a> From<Address<'a>> for super::Address<'a> {
        fn from(a: Address<'a>) -> Self {
            a.0
        }
    }

    impl<'a, 'b: 'a> Decode<'b> for Address<'a> {
        type Error = container::Error<super::Error>;

        fn decode(d: &mut tinycbor::Decoder<'b>) -> Result<Self, Self::Error> {
            let bytes = <&'b [u8]>::decode(d)?;
            if let Some(first) = bytes.first()
                && (first >> 4) == 0b1000
            {
                Decode::decode(&mut Decoder(bytes))
                    .map_err(|e| container::Error::Content(super::Error::Byron(e)))
                    .map(|a| Address(super::Address::Byron(a)))
            } else {
                crate::shelley::Address::from_bytes::<false>(bytes)
                    .map_err(|e| container::Error::Content(super::Error::Shelley(e)))
                    .map(|a| Address(super::Address::Shelley(a)))
            }
        }
    }
}
