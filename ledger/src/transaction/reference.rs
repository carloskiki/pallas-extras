use crate::transaction::{Id, Index};
use tinycbor_derive::{CborLen, Decode, Encode};

/// A reference to an item in a transaction.
///
/// An item is identified by the transaction [`Id`] and its [`Index`] within that transaction.
/// The concrete item depends on the context. For a transaction input, this references an output in
/// a previous transaction. In governance contexts, this could reference a proposal procedure or a
/// vote in a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Reference {
    /// The ID of the transaction that contains the item.
    pub id: Id,
    /// The index of the item within the transaction.
    pub index: Index,
}

pub(crate) mod codec {
    use super::Reference;
    use mitsein::vec1::Vec1;
    use ref_cast::RefCast;
    use tinycbor_derive::{CborLen, Decode, Encode};

    #[derive(Encode, Decode, CborLen)]
    #[repr(transparent)]
    pub enum Byron {
        #[n(0)]
        Input(#[cbor(with = "tinycbor::Encoded<Reference>")] Reference),
    }

    impl From<Reference> for Byron {
        fn from(input: Reference) -> Self {
            Byron::Input(input)
        }
    }

    #[derive(Encode, Decode, CborLen, ref_cast::RefCast)]
    #[repr(transparent)]
    #[cbor(naked)]
    pub struct ByronReferences(cbor_util::NonEmpty<Vec<Byron>>);

    impl From<ByronReferences> for Vec1<Reference> {
        fn from(value: ByronReferences) -> Self {
            // Safety: `Byron` is `repr(transparent)` over `Input`.
            unsafe { std::mem::transmute::<Vec1<Byron>, Vec1<Reference>>(value.0.0) }
        }
    }

    impl<'a> From<&'a Vec1<Reference>> for &'a ByronReferences {
        fn from(value: &'a Vec1<Reference>) -> Self {
            // Safety: `Byron` is `repr(transparent)` over `Input`.
            let vec_byron =
                unsafe { std::mem::transmute::<&'a Vec1<Reference>, &'a Vec1<Byron>>(value) };
            let non_empty = cbor_util::NonEmpty::ref_cast(vec_byron);
            ByronReferences::ref_cast(non_empty)
        }
    }
}
