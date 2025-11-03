use crate::constant::Constant;
use super::builtin;
use macro_rules_attribute::apply;

#[apply(builtin)]
pub fn length(arr: Box<[Constant]>) -> rug::Integer {
    rug::Integer::from(arr.len())
}

#[apply(builtin)]
pub fn index(mut arr: Box<[Constant]>, index: rug::Integer) -> Option<Constant> {
    let index = index.to_usize()?;
    arr.get_mut(index).map(std::mem::take)
}
