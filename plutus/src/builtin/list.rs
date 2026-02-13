use crate::{
    constant::{self, Array, Constant, List},
    machine::Value,
};

pub fn choose<'a>(list: List<'_>, empty: Value<'a>, then: Value<'a>) -> Value<'a> {
    if list.is_empty() { empty } else { then }
}

// We deviate from the builtin spec here. This should not fail once type checking is done, but we
// don't do type checking before calling builtin functions. This is a rare case where we actually
// have to ensure that the types are correct. So this is "failing" in our implementation, whereas
// the spec does not define this as failing.
pub fn mk_cons<'a>(
    head: Constant<'a>,
    tail: List<'a>,
    arena: &'a constant::Arena,
) -> Option<List<'a>> {
    match (head, tail) {
        (Constant::Integer(integer), List::Integer(integers)) => Some(List::Integer(
            arena.integers(
                std::iter::once(integer)
                    .chain(integers.iter())
                    .cloned()
                    .collect::<Vec<_>>(),
            ),
        )),
        (Constant::Bytes(items), List::Bytes(i)) => Some(List::Bytes(
            arena.slice_fill(
                std::iter::once(items)
                    .chain(i.iter().copied())
                    .collect::<Vec<_>>(),
            ),
        )),
        (Constant::String(s), List::String(items)) => Some(List::String(
            arena.slice_fill(
                std::iter::once(s)
                    .chain(items.iter().copied())
                    .collect::<Vec<_>>(),
            ),
        )),
        (Constant::Unit, List::Unit(items)) => Some(List::Unit(
            arena.slice_fill(
                std::iter::once(())
                    .chain(items.iter().copied())
                    .collect::<Vec<_>>(),
            ),
        )),
        (Constant::Boolean(b), List::Boolean(items)) => Some(List::Boolean(
            arena.slice_fill(
                std::iter::once(b)
                    .chain(items.iter().copied())
                    .collect::<Vec<_>>(),
            ),
        )),
        (Constant::Data(data), List::Data(datas)) => Some(List::Data(
            arena.datas(
                std::iter::once(data.clone())
                    .chain(datas.iter().cloned())
                    .collect::<Vec<_>>(),
            ),
        )),
        (Constant::BLSG1Element(projective), List::BLSG1Element(projectives)) => {
            Some(List::BLSG1Element(
                arena.slice_fill(
                    std::iter::once(*projective)
                        .chain(projectives.iter().copied())
                        .collect::<Vec<_>>(),
                ),
            ))
        }
        (Constant::BLSG2Element(projective), List::BLSG2Element(projectives)) => {
            Some(List::BLSG2Element(
                arena.slice_fill(
                    std::iter::once(*projective)
                        .chain(projectives.iter().copied())
                        .collect::<Vec<_>>(),
                ),
            ))
        }
        (Constant::MillerLoopResult(head), List::MillerLoopResult(items)) => {
            Some(List::MillerLoopResult(
                arena.slice_fill(
                    std::iter::once(*head)
                        .chain(items.iter().copied())
                        .collect::<Vec<_>>(),
                ),
            ))
        }
        (Constant::Pair(Constant::Data(data0), Constant::Data(data1)), List::PairData(items)) => {
            Some(List::PairData(
                arena.pair_data(
                    std::iter::once(((*data0).clone(), (*data1).clone()))
                        .chain(items.iter().cloned())
                        .collect::<Vec<_>>(),
                ),
            ))
        }
        (c, List::Generic(Err(ty))) if c.type_of(arena) == ty => {
            Some(List::Generic(Ok(std::slice::from_ref(arena.alloc(c)))))
        }
        (c, List::Generic(Ok(items)))
            if c.type_of(arena) == items.first().expect("non-empty list").type_of(arena) =>
        {
            Some(List::Generic(Ok(arena.slice_fill(
                std::iter::once(c)
                    .chain(items.iter().cloned())
                    .collect::<Vec<_>>(),
            ))))
        }

        _ => return None,
    }
}

pub fn head<'a>(list: List<'a>, arena: &'a constant::Arena) -> Option<Constant<'a>> {
    match list {
        List::Integer(x) => x.first().map(Constant::Integer),
        List::Bytes(x) => x.first().copied().map(Constant::Bytes),
        List::String(x) => x.first().copied().map(Constant::String),
        List::Unit(x) => x.first().map(|_| Constant::Unit),
        List::Boolean(x) => x.first().copied().map(Constant::Boolean),
        List::Data(x) => x.first().map(Constant::Data),
        List::PairData(x) => x.first().map(|(a, b)| {
            Constant::Pair(
                arena.alloc(Constant::Data(a)),
                arena.alloc(Constant::Data(b)),
            )
        }),
        List::BLSG1Element(x) => x.first().map(Constant::BLSG1Element),
        List::BLSG2Element(x) => x.first().map(Constant::BLSG2Element),
        List::MillerLoopResult(x) => x.first().map(Constant::MillerLoopResult),
        List::Generic(Ok(x)) => Some(*x.first().expect("non-empty list")),
        List::Generic(Err(_)) => None,
    }
}

pub fn tail<'a>(list: List<'a>, arena: &'a constant::Arena) -> Option<List<'a>> {
    match list {
        List::Integer(integers) => integers.get(1..).map(List::Integer),
        List::Bytes(items) => items.get(1..).map(List::Bytes),
        List::String(items) => items.get(1..).map(List::String),
        List::Unit(items) => items.get(1..).map(List::Unit),
        List::Boolean(items) => items.get(1..).map(List::Boolean),
        List::Data(datas) => datas.get(1..).map(List::Data),
        List::PairData(items) => items.get(1..).map(List::PairData),
        List::BLSG1Element(projectives) => projectives.get(1..).map(List::BLSG1Element),
        List::BLSG2Element(projectives) => projectives.get(1..).map(List::BLSG2Element),
        List::MillerLoopResult(items) => items.get(1..).map(List::MillerLoopResult),
        List::Generic(Ok(list)) => {
            let tail = list.get(1..)?;
            Some(if tail.is_empty() {
                List::Generic(Err(list.first().expect("non-empty list").type_of(arena)))
            } else {
                List::Generic(Ok(tail))
            })
        }
        List::Generic(Err(_)) => None,
    }
}

pub fn null(list: List<'_>) -> bool {
    list.is_empty()
}

pub fn drop<'a>(count: &rug::Integer, list: List<'a>, arena: &'a constant::Arena) -> List<'a> {
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
        List::Bytes(items) => drop!(items, Bytes),
        List::String(items) => drop!(items, String),
        List::Unit(items) => drop!(items, Unit),
        List::Boolean(items) => drop!(items, Boolean),
        List::Data(datas) => drop!(datas, Data),
        List::PairData(items) => drop!(items, PairData),
        List::BLSG1Element(projectives) => drop!(projectives, BLSG1Element),
        List::BLSG2Element(projectives) => drop!(projectives, BLSG2Element),
        List::MillerLoopResult(items) => drop!(items, MillerLoopResult),
        List::Generic(Ok(list)) => {
            let remainder = list.get(count..).unwrap_or(&[]);
            if remainder.is_empty() {
                List::Generic(Err(list
                    .first()
                    .expect("non-empty list invariant should hold")
                    .type_of(arena)))
            } else {
                List::Generic(Ok(remainder))
            }
        }
        List::Generic(Err(_)) => list,
    }
}

pub fn to_array(list: List<'_>) -> Array<'_> {
    list
}
