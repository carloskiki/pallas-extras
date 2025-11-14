use macro_rules_attribute::apply;
use rug::Integer;

use super::builtin;
use crate::{
    constant::List,
    data::{Construct, Data},
    program::evaluate::Value,
};

#[apply(builtin)]
pub fn choose(
    data: Data,
    construct: Value,
    map: Value,
    list: Value,
    integer: Value,
    bytes: Value,
) -> Value {
    match data {
        Data::Construct(_) => construct,
        Data::Map(_) => map,
        Data::List(_) => list,
        Data::Integer(_) => integer,
        Data::Bytes(_) => bytes,
    }
}

#[apply(builtin)]
pub fn construct(tag: Integer, mut fields: Vec<Data>) -> Data {
    Data::Construct(Construct {
        // We wrap here because this case is quite degenerate.
        // Although the spec strongly suggests using integers that fit in 64 bits,
        // this is not forbidden, although deserialization will fail.
        tag: tag.to_u64_wrapping(),
        value: fields,
    })
}

#[apply(builtin)]
pub fn map(mut pairs: Vec<(Data, Data)>) -> Data {
    Data::Map(pairs)
}

#[apply(builtin)]
pub fn list(mut elements: Vec<Data>) -> Data {
    Data::List(elements)
}

#[apply(builtin)]
pub fn integer(i: Integer) -> Data {
    Data::Integer(i)
}

#[apply(builtin)]
pub fn bytes(b: Vec<u8>) -> Data {
    Data::Bytes(b)
}

#[apply(builtin)]
pub fn un_construct(data: Data) -> Option<(Integer, Vec<Data>)> {
    if let Data::Construct(Construct { tag, mut value }) = data {
        Some((Integer::from(tag), value))
    } else {
        None
    }
}

#[apply(builtin)]
pub fn un_map(data: Data) -> Option<Vec<(Data, Data)>> {
    if let Data::Map(pairs) = data {
        Some(pairs)
    } else {
        None
    }
}

#[apply(builtin)]
pub fn un_list(data: Data) -> Option<Vec<Data>> {
    if let Data::List(mut elements) = data {
        Some(elements)
    } else {
        None
    }
}

#[apply(builtin)]
pub fn un_integer(data: Data) -> Option<Integer> {
    if let Data::Integer(i) = data {
        Some(i)
    } else {
        None
    }
}

#[apply(builtin)]
pub fn un_bytes(data: Data) -> Option<Vec<u8>> {
    if let Data::Bytes(b) = data {
        Some(b)
    } else {
        None
    }
}

#[apply(builtin)]
pub fn equals(data1: Data, data2: Data) -> bool {
    data1 == data2
}

#[apply(builtin)]
pub fn mk_pair(first: Data, second: Data) -> (Data, Data) {
    (first, second)
}

#[apply(builtin)]
pub fn mk_nil(_u: ()) -> List {
    List::empty(crate::constant::Type::Data)
}

#[apply(builtin)]
pub fn mk_nil_pair(_u: ()) -> List {
    List::empty(crate::constant::Type::Pair(Box::new((
        crate::constant::Type::Data,
        crate::constant::Type::Data,
    ))))
}

#[apply(builtin)]
pub fn serialize(data: Data) -> Vec<u8> {
    minicbor::to_vec(data).expect("serialization should not fail")
}
