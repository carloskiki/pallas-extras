use crate::{
    constant::{Array, Constant, List},
    machine::Value,
};

pub fn choose<'a>(list: List<'_>, empty: Value<'a>, then: Value<'a>) -> Value<'a> {
    if list.is_empty() { empty } else { then }
}

// We deviate from the builtin spec here. This should not fail once type checking is done, but we
// don't do type checking before calling builtin functions. This is a rare case where we actually
// have to ensure that the types are correct. So this is "failing" in our implementation, whereas
// the spec does not define this as failing.
pub fn mk_cons(head: Constant, mut tail: List) -> Option<List> {
    Some(match &mut tail.elements {
        Ok(contains) => {
            if std::mem::discriminant(&head) != std::mem::discriminant(&contains[0]) {
                return None;
            }

            contains.push(head);
            tail
        }
        Err(ty) => {
            if &head.type_of() != ty {
                return None;
            }
            tail.elements = Ok(vec![head]);
            tail
        }
    })
}

pub fn head<'a>(list: List<'a>) -> Option<Constant<'a>> {
    todo!()
}

pub fn tail(list: List) -> Option<List> {
    match list.elements {
        Ok(mut list) => {
            let elem = list.pop().expect("non-empty list invariant should hold");

            Some(List {
                elements: if list.is_empty() {
                    Err(elem.type_of())
                } else {
                    Ok(list)
                },
            })
        }
        Err(_) => None,
    }
}

pub fn null(list: List) -> bool {
    list.is_empty()
}

pub fn drop(count: rug::Integer, list: List) -> List {
    if count.is_positive()
        && let Ok(mut contains) = list.elements
    {
        let count = count.to_usize().unwrap_or(usize::MAX);
        if contains.len() > count {
            contains.truncate(contains.len() - count);
            return List {
                elements: Ok(contains),
            };
        } else {
            return List {
                elements: Err(contains[0].type_of()),
            };
        }
    };
    list
}

pub fn to_array(list: List<'_>) -> Array<'_> {
    list
}
