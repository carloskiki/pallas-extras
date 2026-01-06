use tinycbor::{CborLen, Decode, Decoder, Encode, Encoder, Write};

pub mod native;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Address<'a> {
    Shelley(native::Address<'a>),
    Byron(crate::byron::Address<'a>),
}

pub enum Error {
    Shelley(<native::Address<'static> as Decode<'static>>::Error),
    Byron(<crate::byron::Address<'static> as Decode<'static>>::Error),
}

impl Encode for Address<'_> {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        match self {
            Address::Shelley(address) => address.encode(e),
            Address::Byron(address) => address.encode(e),
        }?
    }
}

impl<'a, 'b: 'a> Decode<'b> for Address<'a> {
    type Error = Error;

    fn decode(d: &mut Decoder<'b>) -> Result<Self, Self::Error> {
        let bytes: &[u8] = Decode::decode(d)?;
        
        if bytes.first().is_some_and(|b| (b >> 4) == 0b1000) {
            Address::Byron(Decode::decode(&mut Decoder(bytes)).map_err(Error::Byron)?)
        } else {
            let shelley_address = native::Address::from_bytes(bytes)
                .map_err(|e| Error::Shelley(minicbor::decode::Error::custom(e)))?;
            Ok(Address::Shelley(shelley_address))
        }

        match bytes.first() {
            Some(b) => {
                if (b >> 4) == 0b1000 {
                    let bytes: Box<[u8]> = data.collect();
                    let mut inner_d = Decoder::new(&bytes);
                    Ok(Address::Byron(inner_d.decode()?))
                } else {
                    Ok(Address::Shelley(
                        native::Address::from_bytes(data)
                            .map_err(|e| minicbor::decode::Error::custom(e).at(d.position()))?,
                    ))
                }
            }
            None => Err(minicbor::decode::Error::message("empty byte slice").at(d.position())),
        }
    }
}

impl<C> CborLen<C> for Address {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        match self {
            Address::Shelley(address) => address.cbor_len(ctx),
            Address::Byron(address) => {
                let len = address.cbor_len(ctx);
                len.cbor_len(ctx) + len
            }
        }
    }
}
