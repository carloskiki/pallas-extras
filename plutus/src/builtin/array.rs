use crate::constant::Constant;

pub fn length(arr: &[Constant]) -> rug::Integer {
    rug::Integer::from(arr.len())
}

pub fn index(index: &rug::Integer, mut arr: Vec<Constant>) -> Option<Constant> {
    let index = index.to_usize()?;
    arr.get_mut(index).map(std::mem::take)
}
