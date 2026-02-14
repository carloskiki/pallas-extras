use crate::constant::{Array, Constant, List};

pub fn length(arr: Array<'_>) -> rug::Integer {
    match arr.0 {
        List::Integer(integers) => integers.len(),
        List::Data(datas) => datas.len(),
        List::PairData(items) => items.len(),
        List::BLSG1Element(projectives) => projectives.len(),
        List::BLSG2Element(projectives) => projectives.len(),
        List::Generic(Ok(items)) => items.len().get(),
        List::Generic(Err(_)) => 0,
    }
    .into()
}

pub fn index<'a>(arr: Array<'a>, index: &rug::Integer) -> Option<Constant<'a>> {
    let index = index.to_usize()?;
    match arr.0 {
        List::Integer(integers) => integers.get(index).map(Constant::Integer),
        List::Data(datas) => datas.get(index).map(Constant::Data),
        List::PairData(items) => items.get(index).map(Constant::PairData),
        List::BLSG1Element(projectives) => projectives.get(index).map(Constant::BLSG1Element),
        List::BLSG2Element(projectives) => projectives.get(index).map(Constant::BLSG2Element),
        List::Generic(Ok(slice)) => slice.get(index).copied(),
        List::Generic(Err(_)) => None,
    }
}
