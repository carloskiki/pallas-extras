use crate::{allegra, alonzo, babbage, byron, conway, mary, shelley};
use tinycbor::Encoded;
use tinycbor_derive::{CborLen, Decode, Encode};

/// Era-independent transaction.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub enum Transaction<'a> {
    #[n(0)]
    Byron(#[cbor(with = "codec::Codec<'a>")] byron::Transaction<'a>),
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
    use mitsein::vec1::Vec1;
    use tinycbor_derive::{CborLen, Decode, Encode};

    #[repr(transparent)]
    #[derive(CborLen, Encode, Decode)]
    pub enum Codec<'a> {
        #[n(0)]
        MempoolTx(byron::Transaction<'a>),
    }

    #[derive(Encode, Decode, CborLen)]
    struct TransactionCodec<'a>(
        byron::Transaction<'a>,
        #[cbor(with = "cbor_util::NonEmpty<Vec<byron::transaction::Witness<'a>>>")]
        Vec1<byron::transaction::Witness<'a>>,
    );

    impl<'a> From<Codec<'a>> for byron::Transaction<'a> {
        fn from(value: Codec<'a>) -> Self {
            match value {
                Codec::MempoolTx(tx) => tx,
            }
        }
    }

    impl<'a> From<&byron::Transaction<'a>> for &Codec<'a> {
        fn from(value: &byron::Transaction<'a>) -> Self {
            // Safety: `Codec` is `repr(transparent)` over `byron::Transaction`.
            unsafe { &*(value as *const byron::Transaction<'a> as *const Codec<'a>) }
        }
    }
}
