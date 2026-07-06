use tinycbor_derive::{CborLen, Decode, Encode};
use crate::transaction::{Id, Index};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
pub struct Input {
    pub id: Id,
    pub index: Index,
}

pub(crate) mod codec {
    use super::Input;
    use ref_cast::RefCast;
    use tinycbor_derive::{CborLen, Decode, Encode};
    use mitsein::vec1::Vec1;

    #[derive(Encode, Decode, CborLen)]
    #[repr(transparent)]
    pub enum Byron {
        #[n(0)]
        Input(#[cbor(with = "tinycbor::Encoded<Input>")] Input),
    }

    impl<'a> From<Input> for Byron {
        fn from(input: Input) -> Self {
            Byron::Input(input)
        }
    }

    #[derive(Encode, Decode, CborLen, ref_cast::RefCast)]
    #[repr(transparent)]
    #[cbor(naked)]
    pub struct ByronInputs(cbor_util::NonEmpty<Vec<Byron>>);

    impl From<ByronInputs> for Vec1<Input> {
        fn from(value: ByronInputs) -> Self {
            // Safety: `Byron` is `repr(transparent)` over `Input`.
            unsafe { std::mem::transmute::<Vec1<Byron>, Vec1<Input>>(value.0.0) }
        }
    }

    impl<'a, 'b> From<&'a Vec1<Input>> for &'a ByronInputs {
        fn from(value: &'a Vec1<Input>) -> Self {
            // Safety: `Byron` is `repr(transparent)` over `Input`.
            let vec_byron = unsafe { std::mem::transmute::<&'a Vec1<Input>, &'a Vec1<Byron>>(value) };
            let non_empty = cbor_util::NonEmpty::ref_cast(vec_byron);
            ByronInputs::ref_cast(non_empty)
        }
    }
}
