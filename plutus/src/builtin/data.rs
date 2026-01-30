use crate::{Construct, Data, constant::List, machine::Value};
use rug::Integer;

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

pub fn construct(tag: Integer, fields: Vec<Data>) -> Data {
    Data::Construct(Construct {
        // We wrap here because this case is quite degenerate.
        // Although the spec strongly suggests using integers that fit in 64 bits,
        // this is not forbidden, although deserialization will fail.
        tag: tag.to_u64_wrapping(),
        value: fields,
    })
}

pub fn map(pairs: Vec<(Data, Data)>) -> Data {
    Data::Map(pairs)
}

pub fn list(elements: Vec<Data>) -> Data {
    Data::List(elements)
}

pub fn integer(i: Integer) -> Data {
    Data::Integer(i)
}

pub fn bytes(b: Vec<u8>) -> Data {
    Data::Bytes(b)
}

pub fn un_construct(data: Data) -> Option<(Integer, Vec<Data>)> {
    if let Data::Construct(Construct { tag, value }) = data {
        Some((Integer::from(tag), value))
    } else {
        None
    }
}

pub fn un_map(data: Data) -> Option<Vec<(Data, Data)>> {
    if let Data::Map(pairs) = data {
        Some(pairs)
    } else {
        None
    }
}

pub fn un_list(data: Data) -> Option<Vec<Data>> {
    if let Data::List(elements) = data {
        Some(elements)
    } else {
        None
    }
}

pub fn un_integer(data: Data) -> Option<Integer> {
    if let Data::Integer(i) = data {
        Some(i)
    } else {
        None
    }
}

pub fn un_bytes(data: Data) -> Option<Vec<u8>> {
    if let Data::Bytes(b) = data {
        Some(b)
    } else {
        None
    }
}

pub fn equals(data1: Data, data2: Data) -> bool {
    data1 == data2
}

pub fn mk_pair(first: Data, second: Data) -> (Data, Data) {
    (first, second)
}

pub fn mk_nil(_: ()) -> List {
    List::empty(crate::constant::Type::Data)
}

pub fn mk_nil_pair(_: ()) -> List {
    List::empty(crate::constant::Type::Pair(Box::new((
        crate::constant::Type::Data,
        crate::constant::Type::Data,
    ))))
}

pub fn serialize(data: Data) -> Vec<u8> {
    tinycbor::to_vec(&data)
}
