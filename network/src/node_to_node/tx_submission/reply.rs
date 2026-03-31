use ledger::Transaction;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct Transactions<'a>(
    #[cbor(with = "cbor_util::Indefinite<Vec<Transaction<'a>>>")] pub Vec<Transaction<'a>>,
);

impl crate::Message for Transactions<'_> {
    const TAG: u64 = 3;
    type ToState = super::Idle;
}

mod ids {
    use ledger::transaction;
    use tinycbor::{
        CborLen, Decode, Encode,
        container::{self, bounded},
        primitive,
    };
    use tinycbor_derive::Decode;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct Ids<'a>(pub Vec<(transaction::Id<'a>, u32)>);

    impl Encode for Ids<'_> {
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

    impl CborLen for Ids<'_> {
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

    impl<'a> Decode<'a> for Ids<'a> {
        type Error = container::Error<bounded::Error<Error>>;

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
pub use ids::Ids;

impl crate::Message for Ids<'_> {
    const TAG: u64 = 1;

    type ToState = super::Idle;
}
