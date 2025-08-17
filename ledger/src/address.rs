use minicbor::{CborLen, Decode, Decoder, Encode};

pub mod byron;
pub mod shelley;

/// A Byron or Shelley Era address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Address {
    Shelley(shelley::Address),
    Byron(byron::Address),
}

impl<C> Encode<C> for Address {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Address::Shelley(address) => e.encode(address),
            Address::Byron(address) => {
                e.bytes_len(address.cbor_len(ctx) as u64)?;
                e.encode(address)
            }
        }?
        .ok()
    }
}

impl<C> Decode<'_, C> for Address {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        // This ignores decoding errors of the inner slices, but does not matter because if the
        // inner slice errors then the value wont parse correctly anyway.
        let mut data = d.bytes_iter()?.flatten().flatten().copied().peekable();

        match data.peek() {
            Some(b) => {
                if (b >> 4) == 0b1000 {
                    let bytes: Box<[u8]> = data.collect();
                    let mut inner_d = Decoder::new(&bytes);
                    Ok(Address::Byron(inner_d.decode()?))
                } else {
                    Ok(Address::Shelley(
                        shelley::Address::from_bytes(data)
                            .map_err(minicbor::decode::Error::custom)?,
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
