use crate::{allegra, alonzo, babbage, byron, conway, mary, shelley};
use tinycbor::Encoded;
use tinycbor_derive::{CborLen, Decode, Encode};

mod id;
pub use id::Id;

/// Era-independent transaction.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Transaction<'a> {
    #[n(0)]
    Byron(#[cbor(with = "codec::Codec<'a>")] byron::transaction::Payload<'a>),
    #[n(1)]
    Shelley(#[cbor(with = "Encoded<shelley::Transaction<'a>>")] shelley::Transaction<'a>),
    #[n(2)]
    Allegra(#[cbor(with = "Encoded<allegra::Transaction<'a>>")] allegra::Transaction<'a>),
    #[n(3)]
    Mary(#[cbor(with = "Encoded<mary::Transaction<'a>>")] mary::Transaction<'a>),
    #[n(4)]
    Alonzo(#[cbor(with = "Encoded<alonzo::Transaction<'a>>")] alonzo::Transaction<'a>),
    #[n(5)]
    Babbage(#[cbor(with = "Encoded<babbage::Transaction<'a>>")] babbage::Transaction<'a>),
    #[n(6)]
    Conway(#[cbor(with = "Encoded<conway::Transaction<'a>>")] conway::Transaction<'a>),
}

mod codec {
    use crate::byron;
    use tinycbor_derive::{CborLen, Decode, Encode};

    #[repr(transparent)]
    #[derive(CborLen, Encode, Decode)]
    pub enum Codec<'a> {
        // We only implement "mempool" transactions for byron because we don't expect to receive
        // payloads that communicate transactions for that era anyway. In the byron era, there were
        // other types of payloads: certificate, update, and vote.
        // See https://github.com/IntersectMBO/cardano-ledger/issues/5124.
        #[n(0)]
        MempoolTx(byron::transaction::Payload<'a>),
    }

    impl<'a> From<Codec<'a>> for byron::transaction::Payload<'a> {
        fn from(codec: Codec<'a>) -> Self {
            match codec {
                Codec::MempoolTx(tx) => tx,
            }
        }
    }

    impl From<&byron::transaction::Payload<'_>> for &Codec<'_> {
        fn from(tx: &byron::transaction::Payload<'_>) -> Self {
            // SAFETY: `Codec` is `repr(transparent)` and has the same layout as `byron::transaction::Payload`.
            unsafe { &*(tx as *const byron::transaction::Payload<'_> as *const Codec<'_>) }
        }
    }
}
