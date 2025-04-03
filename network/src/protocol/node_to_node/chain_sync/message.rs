use minicbor::{Decode, Encode};

use crate::{
    traits::{self, message::{nop_codec, Message}}, Point, Tip
};

use super::state::{CanAwait, Idle, Intersect, MustReply};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Next;

impl Message for Next {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 0;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = CanAwait;
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[cbor(transparent)]
pub struct FindIntersect {
    #[cbor(with = "cbor_util::boxed_slice")]
    points: Box<[Point]>,
}

impl Message for FindIntersect {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 4;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Intersect;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Done;

impl Message for Done {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 7;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = traits::state::Done;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IntersectFound {
    point: Point,
    tip: Tip,
}

impl<C> Encode<C> for IntersectFound {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.encode(self.point)?.encode(self.tip)?.ok()
    }
}

impl<C> Decode<'_, C> for IntersectFound {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let point: Point = d.decode()?;
        let tip: Tip = d.decode()?;
        
        Ok(IntersectFound { point, tip })
    }
}

impl Message for IntersectFound {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 5;
    const ELEMENT_COUNT: u64 = 2;

    type ToState = Idle;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[cbor(transparent)]
pub struct IntersectNotFound {
    tip: Tip,
}

impl Message for IntersectNotFound {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 6;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Idle;
}

// 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollForward {
    header: Box<ledger::block::Header>,
    tip: Tip,
}

impl<C> Encode<C> for RollForward {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.encode(&self.header)?.encode(self.tip)?.ok()
    }
}

impl<C> Decode<'_, C> for RollForward {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let header: Box<ledger::block::Header> = d.decode()?;
        let tip: Tip = d.decode()?;
        
        Ok(RollForward { header, tip })
    }
}

impl Message for RollForward {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 2;
    const ELEMENT_COUNT: u64 = 2;

    type ToState = Idle;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RollBackward {
    point: Point,
    tip: Tip,
}

impl<C> Encode<C> for RollBackward {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.encode(self.point)?.encode(self.tip)?.ok()
    }
}

impl<C> Decode<'_, C> for RollBackward {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let point: Point = d.decode()?;
        let tip: Tip = d.decode()?;
        
        Ok(RollBackward { point, tip })
    }
}

impl Message for RollBackward {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 3;
    const ELEMENT_COUNT: u64 = 2;

    type ToState = Idle;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AwaitReply;

impl Message for AwaitReply {
    const SIZE_LIMIT: usize = 65535;
    const TAG: u8 = 1;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = MustReply;
}

nop_codec!(Next, Done, AwaitReply);
