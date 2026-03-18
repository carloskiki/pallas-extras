use tinycbor::Encode;

use super::state::State;

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
    encoder.array(M::ELEMENT_COUNT);
    M::TAG.encode(&mut encoder);
    message.encode(&mut encoder);
}
