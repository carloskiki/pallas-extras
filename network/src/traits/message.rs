use crate::{
    Encoded,
    traits::state::State,
    typefu::coproduct::{CNil, Coproduct},
};
use bytes::Bytes;
use tinycbor::Encode;

pub trait Message {
    const SIZE_LIMIT: usize;
    const TAG: u64;
    const ELEMENT_COUNT: u64;

    type ToState: State;
}

pub fn encode_message<M>(message: &M, buffer: &mut Vec<u8>)
where
    M: Message + Encode,
{
    let mut encoder = tinycbor::Encoder(buffer);
    encoder.array(M::ELEMENT_COUNT as usize);
    M::TAG.encode(&mut encoder);
    message.encode(&mut encoder);
}

/// Decode a coproduct of `Encoded<M>` with the tag of the message.
pub(crate) trait LazyDecode: Sized {
    fn lazy_decode(bytes: Bytes, tag: u64) -> Option<Self>;
}

impl<M: Message, Tail: LazyDecode> LazyDecode for Coproduct<Encoded<M>, Tail> {
    fn lazy_decode(bytes: Bytes, tag: u64) -> Option<Self> {
        if tag == M::TAG {
            return Some(Coproduct::Inl(Encoded {
                bytes,
                _phantom: std::marker::PhantomData,
            }));
        }

        Tail::lazy_decode(bytes, tag).map(Coproduct::Inr)
    }
}

impl<M: Message> LazyDecode for Coproduct<Encoded<M>, CNil> {
    fn lazy_decode(bytes: Bytes, tag: u64) -> Option<Self> {
        if tag != M::TAG {
            return None;
        }

        Some(Coproduct::Inl(Encoded {
            bytes,
            _phantom: std::marker::PhantomData,
        }))
    }
}
