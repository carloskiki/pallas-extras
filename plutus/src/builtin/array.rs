use crate::constant::{Array, Constant};

pub fn length(arr: Array) -> rug::Integer {
    rug::Integer::from(match arr.elements {
        Ok(contains) => contains.len(),
        Err(_) => 0,
    })
}

pub fn index(arr: Array, index: rug::Integer) -> Option<Constant> {
    let index = index.to_usize()?;
    match arr.elements {
        Ok(mut contains) => Some(std::mem::replace(contains.get_mut(index)?, Constant::Unit)),
        Err(_) => None,
    }
}
