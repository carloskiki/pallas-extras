use macro_rules_attribute::apply;

use super::builtin;
use crate::{constant::Constant, program::evaluate::Value};

#[apply(builtin)]
pub fn choose(list: Vec<Constant>, empty: Value, then: Value) -> Value {
    if list.is_empty() { empty } else { then }
}

// We deviate from the builtin spec here. This should not fail once type checking is done, but we
// don't do type checking before calling builtin functions. This is a rare case where we actually
// have to ensure that the types are correct. So this is "failing" in our implementation, whereas
// the spec does not define this as failing.
#[apply(builtin)]
pub fn mk_cons(head: Constant, mut tail: Vec<Constant>) -> Option<Vec<Constant>> {
    // We know that all elements in the list are of the same type.
    if let Some(last) = tail.last()
        && std::mem::discriminant(&head) != std::mem::discriminant(last)
    {
        return None;
    }

    tail.push(head);
    Some(tail)
}

#[apply(builtin)]
pub fn head(mut list: Vec<Constant>) -> Option<Constant> {
    list.pop()
}

#[apply(builtin)]
pub fn tail(mut list: Vec<Constant>) -> Option<Vec<Constant>> {
    if list.is_empty() {
        None
    } else {
        list.truncate(list.len() - 1);
        Some(list)
    }
}

#[apply(builtin)]
pub fn null(list: Vec<Constant>) -> bool {
    list.is_empty()
}

#[apply(builtin)]
pub fn drop(count: rug::Integer, mut list: Vec<Constant>) -> Vec<Constant> {
    if count.is_negative() {
        return list;
    }

    count.to_usize_wrapping();
    list.truncate(list.len().saturating_sub(count.to_usize_wrapping()));
    list
}

#[apply(builtin)]
pub fn to_array(mut list: Vec<Constant>) -> Box<[Constant]> {
    list.reverse();
    list.into_boxed_slice()
}
