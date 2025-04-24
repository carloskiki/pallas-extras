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
    pub points: Box<[Point]>,
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
    pub point: Point,
    pub tip: Tip,
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
    pub tip: Tip,
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
    pub header: Box<ledger::block::Header>,
    pub tip: Tip,
}

impl<C> Encode<C> for RollForward {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let ledger_era = self.header.body.protocol_version.major.era();
        e.array(2)?.encode(ledger_era)?;
        cbor_util::cbor_encoded::encode(&*self.header, e, &mut ())?;
        
        e.encode(self.tip)?.ok()
    }
}

impl<C> Decode<'_, C> for RollForward {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let len = d.array()?;
        if len.is_some_and(|l| l != 2) {
            return Err(minicbor::decode::Error::message(
                "expected array of length 2",
            ));
        }
        let _era = d.decode::<ledger::protocol::Era>()?;
        dbg!(_era);
        let result: Result<Box<ledger::block::Header>, _> = cbor_util::cbor_encoded::decode(d, &mut ());

        dbg!(&result);
        let header = result?;
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
    pub point: Point,
    pub tip: Tip,
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
