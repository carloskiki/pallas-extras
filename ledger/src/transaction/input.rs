use tinycbor_derive::{CborLen, Decode, Encode};
use crate::transaction::{Id, Index};
use hashbrown::Equivalent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Input<'a> {
    pub id: &'a Id,
    pub index: Index,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Owned {
    pub id: Id,
    pub index: Index,
}

impl Equivalent<Input<'_>> for Owned {
    fn equivalent(&self, key: &Input<'_>) -> bool {
        &self.id == key.id && self.index == key.index
    }
}

pub(crate) mod codec {
    use super::Input;
    use ref_cast::RefCast;
    use tinycbor_derive::{CborLen, Decode, Encode};
    use mitsein::vec1::Vec1;

    #[derive(Encode, Decode, CborLen)]
    #[repr(transparent)]
    pub enum Byron<'a> {
        #[n(0)]
        Input(#[cbor(with = "tinycbor::Encoded<Input<'a>>")] Input<'a>),
    }

    impl<'a> From<Input<'a>> for Byron<'a> {
        fn from(input: Input<'a>) -> Self {
            Byron::Input(input)
        }
    }

    #[derive(Encode, Decode, CborLen, ref_cast::RefCast)]
    #[repr(transparent)]
    #[cbor(naked)]
    pub struct ByronInputs<'a>(cbor_util::NonEmpty<Vec<Byron<'a>>>);

    impl From<ByronInputs<'_>> for Vec1<Input<'_>> {
        fn from(value: ByronInputs<'_>) -> Self {
            // Safety: `Byron` is `repr(transparent)` over `Input`.
            unsafe { std::mem::transmute::<Vec1<Byron<'_>>, Vec1<Input<'_>>>(value.0.0) }
        }
    }

    impl<'a, 'b> From<&'a Vec1<Input<'b>>> for &'a ByronInputs<'b> {
        fn from(value: &'a Vec1<Input<'b>>) -> Self {
            // Safety: `Byron` is `repr(transparent)` over `Input`.
            let vec_byron = unsafe { std::mem::transmute::<&'a Vec1<Input<'b>>, &'a Vec1<Byron<'b>>>(value) };
            let non_empty = cbor_util::NonEmpty::ref_cast(vec_byron);
            ByronInputs::ref_cast(non_empty)
        }
    }
}
