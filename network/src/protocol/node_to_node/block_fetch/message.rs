use minicbor::{Decode, Encode};

use crate::{
    Point,
    traits::{message::{Message, nop_codec}, state},
};

use super::state::{Busy, Idle, Streaming};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestRange {
    pub start: Point,
    pub end: Point,
}

impl<C> Encode<C> for RequestRange {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.encode(self.start)?.encode(self.end)?.ok()
    }
}

impl<C> Decode<'_, C> for RequestRange {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        Ok(Self {
            start: d.decode()?,
            end: d.decode()?,
        })
    }
}

impl Message for RequestRange {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 0;
    const ELEMENT_COUNT: u64 = 2;

    type ToState = Busy;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoBlocks;

impl Message for NoBlocks {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 2;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = Idle;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StartBatch;

impl Message for StartBatch {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 3;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = Streaming;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Block();

impl Encode<()> for Block {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        todo!()
    }
}

impl Decode<'_, ()> for Block {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut ()) -> Result<Self, minicbor::decode::Error> {
        todo!()
    }
}

impl Message for Block {
    const SIZE_LIMIT: usize = 2_500_000;
    const TAG: u8 = 4;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = Streaming;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BatchDone;

impl Message for BatchDone {
    // In the spec, this is 2_500_000, but that's absurdly large for nothing
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 5;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = Idle;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Done;

impl Message for Done {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 1;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = state::Done;
}

nop_codec!(NoBlocks, StartBatch, BatchDone, Done);
