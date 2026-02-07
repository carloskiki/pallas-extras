//! Combinators for cost functions.

use zerocopy::{FromBytes, Immutable, KnownLayout};

use crate::cost::Function;


/// Returns the addition of the costs of two functions.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Add<A, B>(pub A, pub B);

impl<I, A, B> Function<I> for Add<A, B>
where
    A: Function<I>,
    B: Function<I>,
{
    fn cost(&self, input: &I) -> i64 {
        self.0.cost(input).saturating_add(self.1.cost(input))
    }
}


/// Returns the maximum of the costs of two functions.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Max<A, B>(pub A, pub B);

impl<I, A, B> Function<I> for Max<A, B>
where
    A: Function<I>,
    B: Function<I>,
{
    fn cost(&self, input: &I) -> i64 {
        self.0.cost(input).max(self.1.cost(input))
    }
}

/// Returns the minimum of the costs of two functions.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Min<A, B>(pub A, pub B);

impl<I, A, B> Function<I> for Min<A, B>
where
    A: Function<I>,
    B: Function<I>,
{
    fn cost(&self, input: &I) -> i64 {
        self.0.cost(input).min(self.1.cost(input))
    }
}

/// Returns the multiplication of the costs of two functions.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Mul<A, B>(pub A, pub B);

impl<I, A, B> Function<I> for Mul<A, B>
where
    A: Function<I>,
    B: Function<I>,
{
    fn cost(&self, input: &I) -> i64 {
        self.0.cost(input).saturating_mul(self.1.cost(input))
    }
}

/// Returns the subtraction of the costs of two functions.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Sub<A, B>(pub A, pub B);

impl<I, A, B> Function<I> for Sub<A, B>
where
    A: Function<I>,
    B: Function<I>,
{
    fn cost(&self, input: &I) -> i64 {
        self.0.cost(input).saturating_sub(self.1.cost(input))
    }
}
