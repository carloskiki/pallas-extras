use std::num::Saturating;

use crate::cost::Function;
use zerocopy::{FromBytes, Immutable, KnownLayout};

pub mod argument;
pub use argument::*;

pub mod ops;
pub use ops::*;

/// `a + b * x`
pub type Affine<X> = Add<Constant, Factor<X>>;
/// `a + b * x + c * y`
pub type Affine2<X, Y> = Add<Affine<X>, Factor<Y>>;
/// `a * x`
pub type Factor<X> = Mul<Constant, X>;

#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Constant {
    pub value: i64,
}

impl<I> Function<I> for Constant {
    fn cost(&self, _: &I) -> i64 {
        self.value
    }
}

#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Divide {
    pub constant: i64,
    pub c00: i64,
    pub c01: i64,
    pub c02: i64,
    pub c10: i64,
    pub c11: i64,
    pub c20: i64,
    pub minimum: i64,
}

impl Function<(rug::Integer, rug::Integer)> for Divide {
    fn cost(&self, inputs: &(rug::Integer, rug::Integer)) -> i64 {
        let x = Saturating(First.cost(&inputs.0));
        let y = Saturating(First.cost(&inputs.1));
        let c00 = Saturating(self.c00);
        let c01 = Saturating(self.c01);
        let c02 = Saturating(self.c02);
        let c10 = Saturating(self.c10);
        let c11 = Saturating(self.c11);
        let c20 = Saturating(self.c20);
        let minimum = Saturating(self.minimum);

        if x < y {
            return self.constant;
        }
        minimum
            .max(c00 + c10 * x + c20 * x * x + c01 * y + c11 * x * y + c02 * y * y)
            .0
    }
}

/// ExpModInteger execution cost function.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct ExpModIntegerExecution {
    pub c00: i64,
    pub c11: i64,
    pub c12: i64,
}

impl Function<(rug::Integer, rug::Integer, rug::Integer)> for ExpModIntegerExecution {
    fn cost(&self, inputs: &(rug::Integer, rug::Integer, rug::Integer)) -> i64 {
        let base = Saturating(First.cost(&inputs.0));
        let exp = Saturating(First.cost(&inputs.1));
        let modulus = Saturating(First.cost(&inputs.2));
        let c00 = Saturating(self.c00);
        let c11 = Saturating(self.c11);
        let c12 = Saturating(self.c12);

        let mut cost = c00 + c11 * exp * modulus + c12 * exp * modulus * modulus;
        if base > modulus {
            cost += cost / Saturating(2);
        }
        cost.0
    }
}

/// IntegerToByteString memory cost function.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct IntegerToByteStringMemory {
    pub affine: Affine<Third>,
}

impl<A, B> Function<(A, rug::Integer, B)> for IntegerToByteStringMemory
where
    First: Function<B>,
{
    fn cost(&self, inputs @ (_, int, _): &(A, rug::Integer, B)) -> i64 {
        if int.is_zero() {
            return self.affine.cost(inputs);
        }

        FirstIntegerAsBytes.cost(int)
    }
}

/// `a + b * x + c * x^2`
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Quadratic<A> {
    pub affine: Affine<A>,
    pub c2: i64,
}

impl<I, A> Function<I> for Quadratic<A>
where
    A: Function<I>,
{
    fn cost(&self, input: &I) -> i64 {
        let Self { affine, c2 } = self;
        let x = Saturating(affine.1.1.cost(input));
        (Saturating(affine.cost(input)) + Saturating(*c2) * x * x).0
    }
}

/// Equality for strings and bytestrings.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct StringEqualsExecution {
    pub constant: i64,
    pub affine: Affine<First>,
}

impl<I> Function<I> for StringEqualsExecution
where
    First: Function<I>,
    Second: Function<I>,
{
    fn cost(&self, inputs: &I) -> i64 {
        let x = First.cost(inputs);
        let y = Second.cost(inputs);
        let Self { constant, affine } = self;
        if x != y {
            *constant
        } else {
            affine.cost(inputs)
        }
    }
}
