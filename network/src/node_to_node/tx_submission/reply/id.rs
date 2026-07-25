use tinycbor_derive::{CborLen, Decode, Encode};
use ledger::transaction;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub enum Id<'a> {
    #[n(0)]
    Byron(#[cbor(with = "codec::Codec<'a>")] &'a transaction::Id),
    #[n(1)]
    Shelley(&'a transaction::Id),
    #[n(2)]
    Allegra(&'a transaction::Id),
    #[n(3)]
    Mary(&'a transaction::Id),
    #[n(4)]
    Alonzo(&'a transaction::Id),
    #[n(5)]
    Babbage(&'a transaction::Id),
    #[n(6)]
    Conway(&'a transaction::Id),
}

mod codec {
    use ledger::transaction::Id;
    use tinycbor_derive::{CborLen, Decode, Encode};

    #[derive(Encode, Decode, CborLen)]
    #[repr(transparent)]
    pub enum Codec<'a> {
        // We only implement `Transaction` ids for the byron era because we don't expect to receive
        // payloads that communicate transactions for that era anyway. In the byron era, there were
        // other types of ids: update id, certificate id, vote id.
        #[n(0)]
        Transaction(&'a Id),
    }

    impl<'a> From<Codec<'a>> for &'a Id {
        fn from(codec: Codec<'a>) -> Self {
            match codec {
                Codec::Transaction(id) => id,
            }
        }
    }

    impl<'a, 'b> From<&'a &'b Id> for &'a Codec<'b> {
        fn from(id: &'a &'b Id) -> Self {
            // Safety: `Codec` is `repr(transparent)` over `Id`.
            unsafe { &*(id as *const &'b Id as *const Codec<'b>) }
        }
    }
}
