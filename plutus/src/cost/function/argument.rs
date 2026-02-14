//! Cost functions that return the cost of one of their arguments.

use std::num::Saturating;

use crate::{constant::List, cost::Function};
use rug::az::SaturatingCast;
use zerocopy::{FromBytes, Immutable, KnownLayout};

/// Function that returns the cost of the first argument.
#[derive(FromBytes, Immutable, KnownLayout)]
pub struct First;

impl Function<rug::Integer> for First {
    fn cost(&self, input: &rug::Integer) -> i64 {
        (std::mem::size_of_val(input.as_limbs()) / 8).max(1) as i64
    }
}

impl Function<&rug::Integer> for First {
    fn cost(&self, input: &&rug::Integer) -> i64 {
        (std::mem::size_of_val(input.as_limbs()) / 8).max(1) as i64
    }
}

impl Function<List<'_>> for First {
    fn cost(&self, input: &List) -> i64 {
        (match input {
            List::Integer(integers) => integers.len(),
            List::Data(datas) => datas.len(),
            List::PairData(items) => items.len(),
            List::BLSG1Element(projectives) => projectives.len(),
            List::BLSG2Element(projectives) => projectives.len(),
            List::Generic(Ok(items)) => items.len().get(),
            List::Generic(Err(_)) => 0,
        }) as i64
    }
}

impl Function<&[rug::Integer]> for First {
    fn cost(&self, input: &&[rug::Integer]) -> i64 {
        input.len() as i64
    }
}

impl Function<Vec<u8>> for First {
    fn cost(&self, input: &Vec<u8>) -> i64 {
        (input.len() as i64 - 1) / 8 + 1
    }
}

impl Function<&[u8]> for First {
    fn cost(&self, input: &&[u8]) -> i64 {
        (input.len() as i64 - 1) / 8 + 1
    }
}

impl Function<String> for First {
    fn cost(&self, input: &String) -> i64 {
        input.chars().count() as i64
    }
}

impl Function<&str> for First {
    fn cost(&self, input: &&str) -> i64 {
        input.chars().count() as i64
    }
}

impl Function<&crate::Data> for First {
    fn cost(&self, inputs: &&crate::Data) -> i64 {
        (Saturating(4)
            + match inputs {
                crate::Data::Map(items) => items.iter().fold(Saturating(0), |a, (k, v)| {
                    Saturating(self.cost(&k)) + Saturating(self.cost(&v)) + a
                }),
                crate::Data::List(datas)
                | crate::Data::Construct(crate::Construct { value: datas, .. }) => datas
                    .iter()
                    .fold(Saturating(0), |a, d| Saturating(self.cost(&d)) + a),
                crate::Data::Bytes(items) => Saturating(self.cost(items)),
                crate::Data::Integer(integer) => Saturating(self.cost(integer)),
            })
        .0
    }
}

impl<X, Y> Function<(X, Y)> for First
where
    First: Function<X>,
{
    fn cost(&self, input: &(X, Y)) -> i64 {
        self.cost(&input.0)
    }
}

impl<X, Y, Z> Function<(X, Y, Z)> for First
where
    First: Function<X>,
{
    fn cost(&self, input: &(X, Y, Z)) -> i64 {
        self.cost(&input.0)
    }
}

/// Function that returns the cost of the second argument.
#[derive(FromBytes, Immutable, KnownLayout)]
pub struct Second;

impl<X, Y> Function<(X, Y)> for Second
where
    First: Function<Y>,
{
    fn cost(&self, input: &(X, Y)) -> i64 {
        First.cost(&input.1)
    }
}

impl<X, Y, Z> Function<(X, Y, Z)> for Second
where
    First: Function<Y>,
{
    fn cost(&self, input: &(X, Y, Z)) -> i64 {
        First.cost(&input.1)
    }
}

/// Function that returns the cost of the third argument.
#[derive(FromBytes, Immutable, KnownLayout)]
pub struct Third;

impl<X, Y, Z> Function<(X, Y, Z)> for Third
where
    First: Function<Z>,
{
    fn cost(&self, input: &(X, Y, Z)) -> i64 {
        First.cost(&input.2)
    }
}

/// Function that returns the integer value of the first argument.
#[derive(FromBytes, Immutable, KnownLayout)]
pub struct FirstInteger;

impl Function<&rug::Integer> for FirstInteger {
    fn cost(&self, input: &&rug::Integer) -> i64 {
        <_ as SaturatingCast<i64>>::saturating_cast(*input).saturating_abs()
    }
}

impl<A, B> Function<(A, B)> for FirstInteger
where
    FirstInteger: Function<A>,
{
    fn cost(&self, input: &(A, B)) -> i64 {
        self.cost(&input.0)
    }
}

impl<A, B, C> Function<(A, B, C)> for FirstInteger
where
    FirstInteger: Function<A>,
{
    fn cost(&self, input: &(A, B, C)) -> i64 {
        self.cost(&input.0)
    }
}

/// Function that returns the integer value of the first argument divided by 8.
#[derive(FromBytes, Immutable, KnownLayout)]
pub struct FirstIntegerAsBytes;

impl Function<&rug::Integer> for FirstIntegerAsBytes {
    fn cost(&self, input: &&rug::Integer) -> i64 {
        let value: i64 = input.saturating_cast();
        value.unsigned_abs().div_ceil(8) as i64
    }
}

impl<A, B> Function<(A, B)> for FirstIntegerAsBytes
where
    FirstIntegerAsBytes: Function<A>,
{
    fn cost(&self, input: &(A, B)) -> i64 {
        self.cost(&input.0)
    }
}

impl<A, B, C> Function<(A, B, C)> for FirstIntegerAsBytes
where
    FirstIntegerAsBytes: Function<A>,
{
    fn cost(&self, input: &(A, B, C)) -> i64 {
        self.cost(&input.0)
    }
}
