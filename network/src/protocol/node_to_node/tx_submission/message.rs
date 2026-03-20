use ledger::{Transaction, transaction};
use tinycbor::{CborLen, Decode, Encode};
use tinycbor_derive::{CborLen, Decode, Encode};

use crate::traits::{message::Message, state};

use super::state::{Idle, TransactionIds};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct Init;

impl Message for Init {
    const SIZE_LIMIT: usize = 5670;
    const TAG: u64 = 6;
    const ELEMENT_COUNT: u64 = 0;

    type ToState = Idle;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct RequestTransactions<'a>(
    #[cbor(with = "cbor_util::Indefinite<Vec<transaction::Id<'a>>>")] pub Vec<transaction::Id<'a>>,
);

impl Message for RequestTransactions<'_> {
    const SIZE_LIMIT: usize = 5760;
    const TAG: u64 = 2;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = super::state::Transactions;
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct ReplyTransactions<'a>(
    #[cbor(with = "cbor_util::Indefinite<Vec<Transaction<'a>>>")] pub Vec<Transaction<'a>>,
);

impl Message for ReplyTransactions<'_> {
    const SIZE_LIMIT: usize = 2_500_000;

    const TAG: u64 = 3;

    const ELEMENT_COUNT: u64 = 1;

    type ToState = super::state::Idle;
}

mod request_transaction_ids {
    use super::*;
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RequestTransactionIds<const BLOCKING: bool> {
        pub acknowledge: u16,
        pub request: u16,
    }

    impl<const B: bool> Encode for RequestTransactionIds<B> {
        fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
            B.encode(e)?;
            self.acknowledge.encode(e)?;
            self.request.encode(e)
        }
    }

    impl<const B: bool> CborLen for RequestTransactionIds<B> {
        fn cbor_len(&self) -> usize {
            B.cbor_len() + self.acknowledge.cbor_len() + self.request.cbor_len()
        }
    }

    #[derive(Decode)]
    #[cbor(naked)]
    struct Codec {
        blocking: bool,
        acknowledge: u16,
        request: u16,
    }

    impl<'a, const B: bool> Decode<'a> for RequestTransactionIds<B> {
        type Error = <Codec as Decode<'a>>::Error;

        fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
            let Codec {
                blocking,
                acknowledge,
                request,
            } = Codec::decode(d)?;
            if blocking != B {
                return Err(Error::Blocking(tinycbor::primitive::Error::InvalidHeader));
            }

            Ok(Self {
                acknowledge,
                request,
            })
        }
    }
}
pub use request_transaction_ids::RequestTransactionIds;

impl Message for RequestTransactionIds<false> {
    const SIZE_LIMIT: usize = 5760;
    const TAG: u64 = 0;
    const ELEMENT_COUNT: u64 = 3;

    type ToState = TransactionIds<false>;
}

impl Message for RequestTransactionIds<true> {
    const SIZE_LIMIT: usize = 5760;
    const TAG: u64 = 0;
    const ELEMENT_COUNT: u64 = 3;

    type ToState = TransactionIds<true>;
}

mod reply_transaction_ids {
    use tinycbor::{container, primitive};

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct ReplyTransactionIds<'a>(pub Vec<(transaction::Id<'a>, u32)>);

    impl Encode for ReplyTransactionIds<'_> {
        fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
            e.begin_array()?;
            for (id, size) in self.0.iter() {
                e.array(2)?;
                id.encode(e)?;
                size.encode(e)?;
            }
            e.end()
        }
    }

    impl CborLen for ReplyTransactionIds<'_> {
        fn cbor_len(&self) -> usize {
            2 + self
                .0
                .iter()
                .map(|(id, size)| 2.cbor_len() + id.cbor_len() + size.cbor_len())
                .sum::<usize>()
        }
    }

    #[derive(Decode)]
    struct Codec<'a> {
        transaction: transaction::Id<'a>,
        size: u32,
    }

    impl<'a> Decode<'a> for ReplyTransactionIds<'a> {
        type Error = <Codec<'a> as Decode<'a>>::Error;

        fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
            let mut ids = Vec::new();
            let mut visitor = d.array_visitor()?;
            if visitor.definite() {
                return Err(container::Error::Malformed(primitive::Error::InvalidHeader));
            }

            while let Some(codec) = visitor.visit::<Codec<'a>>() {
                let Codec { transaction, size } = codec?;
                ids.push((transaction, size));
            }
            Ok(Self(ids))
        }
    }
}
pub use reply_transaction_ids::ReplyTransactionIds;

impl Message for ReplyTransactionIds<'_> {
    const SIZE_LIMIT: usize = 2_500_000;
    const TAG: u64 = 1;
    const ELEMENT_COUNT: u64 = 1;

    type ToState = Idle;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Done;

impl Message for Done {
    // The spec says limit is 2_500_000 bytes, but this is way to big.
    const SIZE_LIMIT: usize = 5760;

    const TAG: u64 = 4;

    const ELEMENT_COUNT: u64 = 0;

    type ToState = state::Done;
}
