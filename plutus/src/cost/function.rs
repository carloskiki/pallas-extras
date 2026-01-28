use zerocopy::{FromBytes, Immutable, KnownLayout};

use crate::cost::Function;

use super::Argument;

#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Constant {
    pub value: i64,
}

impl Function for Constant {
    fn cost<X: Argument, Y: Argument, Z: Argument>(&self, _: X, _: Y, _: Z) -> i64 {
        self.value
    }
}
