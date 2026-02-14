use mitsein::slice1::Slice1;

use crate::{
    constant::{self, Array, Constant, List},
    machine::Value,
};

pub fn choose<'a>(list: List<'_>, empty: Value<'a>, then: Value<'a>) -> Value<'a> {
    if null(list) { empty } else { then }
}

// To have access to the arena, we return a struct that impls `Output`, and actually implements the
// builtin.
pub fn mk_cons<'a>(head: Constant<'a>, tail: List<'a>) -> MkCons<'a> {
    MkCons { head, tail }
}

pub struct MkCons<'a> {
    head: Constant<'a>,
    tail: List<'a>,
}

impl<'a> super::Output<'a> for MkCons<'a> {
    fn into(
        MkCons { head, tail }: Self,
        arena: &'a constant::Arena,
    ) -> Option<crate::machine::Value<'a>> {
        macro_rules! cons {
            ($head:ident, $tail:ident, $variant:ident, $method:ident) => {{
                let list: Vec<_> = std::iter::once($head.clone())
                    .chain($tail.iter().cloned())
                    .collect();
                List::$variant(arena.$method(list))
            }};
        }

        let list = match (head, tail) {
            (Constant::Integer(head), List::Integer(tail)) => cons!(head, tail, Integer, integers),
            (Constant::PairData(head), List::PairData(tail)) => {
                cons!(head, tail, PairData, pair_datas)
            }
            (Constant::Data(head), List::Data(tail)) => cons!(head, tail, Data, datas),
            (Constant::BLSG1Element(head), List::BLSG1Element(tail)) => {
                cons!(head, tail, BLSG1Element, slice_fill)
            }
            (Constant::BLSG2Element(head), List::BLSG2Element(tail)) => {
                cons!(head, tail, BLSG2Element, slice_fill)
            }
            (head, List::Generic(Err(ty))) if head.type_eq(ty) => {
                List::Generic(Ok(mitsein::slice1::from_ref(ty)))
            }
            (head, List::Generic(Ok(tail))) if head.type_eq(tail.first()) => {
                let list: Vec<_> = std::iter::once(head).chain(tail.iter().copied()).collect();
                List::Generic(Ok(Slice1::try_from_slice(arena.slice_fill(list))
                    .expect("can't be non-empty, we just added an element")))
            }

            // Mismatched types.
            _ => return None,
        };
        Some(Value::Constant(Constant::List(list)))
    }
}

pub fn head<'a>(list: List<'a>) -> Option<Constant<'a>> {
    match list {
        List::Integer(x) => x.first().map(Constant::Integer),
        List::Data(x) => x.first().map(Constant::Data),
        List::PairData(x) => x.first().map(Constant::PairData),
        List::BLSG1Element(x) => x.first().map(Constant::BLSG1Element),
        List::BLSG2Element(x) => x.first().map(Constant::BLSG2Element),
        List::Generic(Ok(x)) => Some(*x.first()),
        List::Generic(Err(_)) => None,
    }
}

pub fn tail<'a>(list: List<'a>) -> Option<List<'a>> {
    match list {
        List::Integer(integers) => integers.get(1..).map(List::Integer),
        List::Data(datas) => datas.get(1..).map(List::Data),
        List::PairData(items) => items.get(1..).map(List::PairData),
        List::BLSG1Element(projectives) => projectives.get(1..).map(List::BLSG1Element),
        List::BLSG2Element(projectives) => projectives.get(1..).map(List::BLSG2Element),
        List::Generic(Ok(list)) => {
            let (head, tail) = list.split_first();
            Some(if tail.is_empty() {
                List::Generic(Err(head))
            } else {
                List::Generic(Ok(
                    Slice1::try_from_slice(tail).expect("tail check non-empty")
                ))
            })
        }
        List::Generic(Err(_)) => None,
    }
}

pub fn null(list: List<'_>) -> bool {
    match list {
        List::Integer(integers) => integers.is_empty(),
        List::Data(datas) => datas.is_empty(),
        List::PairData(items) => items.is_empty(),
        List::BLSG1Element(projectives) => projectives.is_empty(),
        List::BLSG2Element(projectives) => projectives.is_empty(),
        List::Generic(non_empty) => non_empty.is_err(),
    }
}

pub fn drop<'a>(count: &rug::Integer, list: List<'a>) -> List<'a> {
    if !count.is_positive() {
        return list;
    };
    let count = count.to_usize().unwrap_or(usize::MAX);
    macro_rules! drop {
        ($variable:ident, $variant:ident) => {
            $variable
                .get(count..)
                .map(List::$variant)
                .unwrap_or(List::$variant(&[]))
        };
    }

    match list {
        List::Integer(integers) => drop!(integers, Integer),
        List::Data(datas) => drop!(datas, Data),
        List::PairData(items) => drop!(items, PairData),
        List::BLSG1Element(projectives) => drop!(projectives, BLSG1Element),
        List::BLSG2Element(projectives) => drop!(projectives, BLSG2Element),
        List::Generic(Ok(list)) => {
            let remainder = list.get(count..).unwrap_or(&[]);
            if remainder.is_empty() {
                List::Generic(Err(list.first()))
            } else {
                List::Generic(Ok(
                    Slice1::try_from_slice(remainder).expect("remainder check non-empty")
                ))
            }
        }
        List::Generic(Err(_)) => list,
    }
}

pub fn to_array(list: List<'_>) -> Array<'_> {
    Array(list)
}
