use crate::byron::transaction;
use tinycbor_derive::{CborLen, Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Id<'a> {
    #[n(0)]
    Byron(&'a transaction::Id),
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
    use tinycbor_derive::{CborLen, Decode, Encode};
    use crate::byron::transaction;
    
    #[derive(Encode, Decode, CborLen)]
    #[repr(transparent)]
    enum Codec<'a> {
        // We only implement `Transaction` ids for the byron era because we don't expect to receive
        // payloads that communicate transactions for that era anyway. In the byron era, there were
        // other types of ids: update id, certificate id, vote id.
        #[n(0)]
        Transaction(&'a transaction::Id),
    }

    impl<'a> From<Codec<'a>> for &'a transaction::Id {
        fn from(codec: Codec<'a>) -> Self {
            match codec {
                Codec::Transaction(id) => id,
            }
        }
    }

    impl<'a, 'b> From<&'b &'a transaction::Id> for &'b Codec<'a> {
        fn from(id: &'b &'a transaction::Id) -> Self {
            // Safety: `Codec` is `repr(transparent)` over `&transaction::Id`.
            unsafe { &*(id as *const &'a transaction::Id as *const Codec<'a>) }
        }
    }
}
