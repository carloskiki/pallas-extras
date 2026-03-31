use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(naked)]
pub struct Transactions<'a>(
    #[cbor(with = "cbor_util::Indefinite<Vec<ledger::transaction::Id<'a>>>")]
    pub  Vec<ledger::transaction::Id<'a>>,
);

impl crate::Message for Transactions<'_> {
    const TAG: u64 = 2;

    type ToState = super::Transactions;
}

mod ids {
    use tinycbor::{CborLen, Decode, Encode};
    use tinycbor_derive::Decode;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Ids<const BLOCKING: bool> {
        pub acknowledge: u16,
        pub request: u16,
    }

    impl<const B: bool> Encode for Ids<B> {
        fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
            B.encode(e)?;
            self.acknowledge.encode(e)?;
            self.request.encode(e)
        }
    }

    impl<const B: bool> CborLen for Ids<B> {
        fn cbor_len(&self) -> usize {
            B.cbor_len() + self.acknowledge.cbor_len() + self.request.cbor_len()
        }
    }

    #[derive(Decode)]
    #[cbor(naked)]
    pub struct Codec {
        blocking: bool,
        acknowledge: u16,
        request: u16,
    }

    impl<'a, const B: bool> Decode<'a> for Ids<B> {
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
pub use ids::Ids;

impl crate::Message for Ids<false> {
    const TAG: u64 = 0;

    type ToState = super::TransactionIds<false>;
}

impl crate::Message for Ids<true> {
    const TAG: u64 = 0;

    type ToState = super::TransactionIds<true>;
}
