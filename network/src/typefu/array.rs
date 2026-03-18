use std::ops::Add;

use hybrid_array::{
    ArraySize,
    sizes::{U0, U1},
};

use crate::typefu::coproduct::{CNil, Coproduct};

pub trait Index {
    type Length: ArraySize;

    fn index(&self) -> usize;
}

impl Index for CNil {
    type Length = U0;

    fn index(&self) -> usize {
        match *self {}
    }
}

impl<L, R: Index<Length: Add<U1, Output: ArraySize>>> Index for Coproduct<L, R> {
    type Length = <R::Length as Add<U1>>::Output;

    fn index(&self) -> usize {
        match self {
            Coproduct::Inl(_) => 0,
            Coproduct::Inr(tail) => 1 + tail.index(),
        }
    }
}
