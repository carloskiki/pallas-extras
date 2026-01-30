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
        let x = First.cost(&inputs.0);
        let y = First.cost(&inputs.1);
        let Self {
            constant,
            c00,
            c01,
            c02,
            c10,
            c11,
            c20,
            minimum,
        } = self;
        if x < y {
            return *constant;
        }
        (*minimum).max(c00 + *c10 * x + c20 * x * x + c01 * y + c11 * x * y + c02 * y * y)
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
        let base = First.cost(&inputs.0);
        let exp = First.cost(&inputs.1);
        let modulus = First.cost(&inputs.2);
        let Self { c00, c11, c12 } = self;
        let mut cost = c00 + c11 * exp * modulus + c12 * exp * modulus * modulus;
        if base > modulus {
            cost += cost / 2;
        }
        cost
    }
}

/// IntegerToByteString memory cost function.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct IntegerToByteStringMemory;

impl<A, B> Function<(A, rug::Integer, B)> for IntegerToByteStringMemory
where
    First: Function<B>,
{
    fn cost(&self, (_, int, third): &(A, rug::Integer, B)) -> i64 {
        if int.is_zero() {
            return First.cost(third);
        }

        use rug::az::SaturatingCast;
        let value: i64 = int.saturating_cast();
        ((value.unsigned_abs() - 1).div_ceil(8) + 1) as i64
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
        let x = affine.1.1.cost(input);
        affine.cost(input) + c2 * x * x
    }
}

/// Equality for strings and bytestrings.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct StringEquals {
    pub constant: i64,
    pub affine: Affine<First>,
}

impl<I> Function<I> for StringEquals
where
    First: Function<I>,
    Second: Function<I>,
{
    fn cost(&self, inputs: &I) -> i64 {
        let x = First.cost(inputs);
        let y = Second.cost(inputs);
        let Self { constant, affine } = self;
        if x == y {
            *constant
        } else {
            affine.cost(inputs)
        }
    }
}

