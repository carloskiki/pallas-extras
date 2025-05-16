use ledger::transaction::Id;
use minicbor::{Decode, Encode};

use crate::traits::{
    message::{Message, nop_codec},
    state,
};

use super::state::{Idle, TransactionIds};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Init;

impl Message for Init {
    const SIZE_LIMIT: usize = 5670;
    const TAG: u8 = 6;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = Idle;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
#[cbor(transparent)]
pub struct RequestTransactions(#[cbor(with = "cbor_util::boxed_slice")] pub Box<[Id]>);

impl Message for RequestTransactions {
    const SIZE_LIMIT: usize = 5760;
    const TAG: u8 = 2;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = super::state::Transactions;
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cbor(transparent)]
pub struct ReplyTransactions(#[cbor(with = "cbor_util::boxed_slice")] pub Box<[ledger::Transaction]>);

impl Message for ReplyTransactions {
    const SIZE_LIMIT: usize = 2_500_000;

    const TAG: u8 = 3;

    const ELEMENT_COUNT: u64 = 1;

    type ToState = super::state::Idle;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestTransactionIds<const BLOCKING: bool> {
    pub acknowledge: u16,
    pub request: u16,
}

impl Message for RequestTransactionIds<true> {
    const SIZE_LIMIT: usize = 5760;
    const TAG: u8 = 0;
    const ELEMENT_COUNT: u64 = 3;

    type ToState = TransactionIds<true>;
}

impl Message for RequestTransactionIds<false> {
    const SIZE_LIMIT: usize = 5760;
    const TAG: u8 = 1;
    const ELEMENT_COUNT: u64 = 3;

    type ToState = TransactionIds<false>;
}

impl<C, const B: bool> Encode<C> for RequestTransactionIds<B> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.bool(B)?.u16(self.acknowledge)?.u16(self.request)?.ok()
    }
}

impl<C, const B: bool> minicbor::Decode<'_, C> for RequestTransactionIds<B> {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        if d.bool()? != B {
            return Err(minicbor::decode::Error::message("Blocking mismatch"));
        }
        Ok(Self {
            acknowledge: d.u16()?,
            request: d.u16()?,
        })
    }
}

pub type TransactionSize = u32;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ReplyTransactionIds(pub Box<[(Id, TransactionSize)]>);

impl Message for ReplyTransactionIds {
    const SIZE_LIMIT: usize = 2_500_000;
    const TAG: u8 = 1;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Idle;
}

impl<C> Encode<C> for ReplyTransactionIds {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(self.0.len() as u64)?;
        for (id, size) in self.0.iter() {
            e.array(2)?;
            minicbor::bytes::encode(id, e, ctx)?;
            e.u32(*size)?;
        }
        Ok(())
    }
}

impl<C> minicbor::Decode<'_, C> for ReplyTransactionIds {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        #[derive(Decode)]
        struct TxIdAndSize {
            #[cbor(n(0), with = "minicbor::bytes")]
            id: Id,
            #[cbor(n(1))]
            size: TransactionSize,
        }

        let vec = d
            .array_iter()?
            .map(|result| result.map(|TxIdAndSize { id, size }| (id, size)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self(vec.into_boxed_slice()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Done;

impl Message for Done {
    // The spec says limit is 2_500_000 bytes, but this is way to big.
    const SIZE_LIMIT: usize = 5760;

    const TAG: u8 = 4;

    const ELEMENT_COUNT: u64 = 0;

    type ToState = state::Done;
}

nop_codec!(Init, Done);
