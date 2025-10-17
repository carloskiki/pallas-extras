use crate::constant::Constant;

pub fn choose(list: &[Constant], empty: u32, then: u32) -> u32 {
    if list.is_empty() { empty } else { then }
}

pub fn mk_cons(head: Constant, mut tail: Vec<Constant>) -> Vec<Constant> {
    tail.push(head);
    tail
}

pub fn head(mut list: Vec<Constant>) -> Option<Constant> {
    list.pop()
}

pub fn tail(mut list: Vec<Constant>) -> Option<Vec<Constant>> {
    if list.is_empty() {
        None
    } else {
        list.truncate(list.len() - 1);
        Some(list)
    }
}

pub fn null(list: &[Constant]) -> bool {
    list.is_empty()
}

pub fn drop(count: &rug::Integer, mut list: Vec<Constant>) -> Vec<Constant> {
    if count.is_negative() {
        return list;
    }
    
    count.to_usize_wrapping();
    list.truncate(list.len().saturating_sub(count.to_usize_wrapping()));
    list
}

pub fn to_array(mut list: Vec<Constant>) -> Vec<Constant> {
    list.reverse();
    list
}
