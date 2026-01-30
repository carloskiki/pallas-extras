use crate::{
    constant::{Array, Constant, List},
    cost::Function,
};
use rug::az::SaturatingCast;
use zerocopy::{FromBytes, Immutable, KnownLayout};

/// Function that returns the cost of the first argument.
#[derive(FromBytes, Immutable, KnownLayout)]
pub struct First;

impl Function<rug::Integer> for First {
    fn cost(&self, input: &rug::Integer) -> i64 {
        (std::mem::size_of_val(input.as_limbs()) / 8) as i64 + 1
    }
}

impl Function<List> for First {
    fn cost(&self, input: &List) -> i64 {
        match &input.elements {
            Ok(elements) => elements.len() as i64,
            Err(_) => 0,
        }
    }
}

impl<T: Into<Constant>> Function<Vec<T>> for First {
    fn cost(&self, input: &Vec<T>) -> i64 {
        input.len() as i64
    }
}

impl Function<Array> for First {
    fn cost(&self, input: &Array) -> i64 {
        match &input.elements {
            Ok(elements) => elements.len() as i64,
            Err(_) => 0,
        }
    }
}

impl Function<Vec<u8>> for First {
    fn cost(&self, input: &Vec<u8>) -> i64 {
        (input.len() as i64 - 1) / 8 + 1
    }
}

impl Function<String> for First {
    fn cost(&self, input: &String) -> i64 {
        input.chars().count() as i64
    }
}

impl Function<crate::Data> for First {
    fn cost(&self, inputs: &crate::Data) -> i64 {
        4 + match inputs {
            crate::Data::Map(items) => items.iter().map(|(k, v)| self.cost(k) + self.cost(v)).sum(),
            crate::Data::List(datas)
            | crate::Data::Construct(crate::Construct { value: datas, .. }) => {
                datas.iter().map(|d| self.cost(d)).sum()
            }
            crate::Data::Bytes(items) => self.cost(items),
            crate::Data::Integer(integer) => self.cost(integer),
        }
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
pub struct FirstValue;

impl Function<rug::Integer> for FirstValue {
    fn cost(&self, input: &rug::Integer) -> i64 {
        <_ as SaturatingCast<i64>>::saturating_cast(input).abs()
    }
}

impl<A, B> Function<(A, B)> for FirstValue
where
    FirstValue: Function<A>,
{
    fn cost(&self, input: &(A, B)) -> i64 {
        self.cost(&input.0)
    }
}

impl<A, B, C> Function<(A, B, C)> for FirstValue
where
    FirstValue: Function<A>,
{
    fn cost(&self, input: &(A, B, C)) -> i64 {
        self.cost(&input.0)
    }
}
