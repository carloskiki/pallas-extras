use macro_rules_attribute::apply;

use crate::{constant::Constant, program::evaluate::Value};
use super::builtin;

#[apply(builtin)]
pub fn choose(list: Vec<Constant>, empty: Value, then: Value) -> Value {
    if list.is_empty() { empty } else { then }
}

// TODO: Ensure same type for head and tail.
#[apply(builtin)]
pub fn mk_cons(head: Constant, mut tail: Vec<Constant>) -> Vec<Constant> {
    tail.push(head);
    tail
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
pub fn to_array(mut list: Vec<Constant>) -> Vec<Constant> {
    list.reverse();
    list
}
