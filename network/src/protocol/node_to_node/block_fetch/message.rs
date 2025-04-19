use minicbor::{Decode, Encode};

use crate::{
    Point,
    traits::{message::{Message, nop_codec}, state},
};

use super::state::{Busy, Idle, Streaming};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestRange<const MAINNET: bool> {
    pub start: Point,
    pub end: Point,
}

impl<C, const M: bool> Encode<C> for RequestRange<M> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.encode(self.start)?.encode(self.end)?.ok()
    }
}

impl<C, const M: bool> Decode<'_, C> for RequestRange<M> {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        Ok(Self {
            start: d.decode()?,
            end: d.decode()?,
        })
    }
}

impl<const M: bool> Message for RequestRange<M> {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 0;
    const ELEMENT_COUNT: u64 = 2;

    type ToState = Busy<M>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoBlocks<const MAINNET: bool>;

impl<const M: bool> Message for NoBlocks<M> {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 2;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = Idle<M>;
}

impl<C, const M: bool> Encode<C> for NoBlocks<M> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        _: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        Ok(())
    }
}

impl<C, const M: bool> Decode<'_, C> for NoBlocks<M> {
    fn decode(
        _: &mut minicbor::Decoder<'_>,
        _: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        Ok(Self)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StartBatch<const MAINNET: bool>;

impl<const M: bool> Message for StartBatch<M> {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 3;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = Streaming<M>;
}

impl<C, const M: bool> Encode<C> for StartBatch<M> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        _: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        Ok(())
    }
}

impl<C, const M: bool> Decode<'_, C> for StartBatch<M> {
    fn decode(
        _: &mut minicbor::Decoder<'_>,
        _: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        Ok(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cbor(transparent)]
pub struct Block<const MAINNET: bool>(pub ledger::block::Block<MAINNET>);

impl<const M: bool> Message for Block<M> {
    const SIZE_LIMIT: usize = 2_500_000;
    const TAG: u8 = 4;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Streaming<M>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BatchDone<const MAINNET: bool>;

impl<const M: bool> Message for BatchDone<M> {
    // In the spec, this is 2_500_000, but that's absurdly large for nothing
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 5;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = Idle<M>;
}

impl<C, const M: bool> Encode<C> for BatchDone<M> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        _: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        Ok(())
    }
}

impl<C, const M: bool> Decode<'_, C> for BatchDone<M> {
    fn decode(
        _: &mut minicbor::Decoder<'_>,
        _: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        Ok(Self)
    }
}


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Done;

impl Message for Done {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 1;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = state::Done;
}

nop_codec!(Done);
