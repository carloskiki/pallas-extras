use crate::{Construct, Data, constant::List, machine::Value};
use rug::Integer;

pub fn choose<'a>(
    data: &Data,
    construct: Value<'a>,
    map: Value<'a>,
    list: Value<'a>,
    integer: Value<'a>,
    bytes: Value<'a>,
) -> Value<'a> {
    match data {
        Data::Construct(_) => construct,
        Data::Map(_) => map,
        Data::List(_) => list,
        Data::Integer(_) => integer,
        Data::Bytes(_) => bytes,
    }
}

pub fn construct(tag: &Integer, fields: Vec<Data>) -> Data {
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

pub fn un_construct(data: &Data) -> Option<(Integer, &[Data])> {
    if let Data::Construct(Construct { tag, value }) = data {
        Some((Integer::from(*tag), value))
    } else {
        None
    }
}

pub fn un_map(data: &Data) -> Option<&[(Data, Data)]> {
    if let Data::Map(pairs) = data {
        Some(pairs)
    } else {
        None
    }
}

pub fn un_list(data: &Data) -> Option<&[Data]> {
    if let Data::List(elements) = data {
        Some(elements)
    } else {
        None
    }
}

pub fn un_integer(data: &Data) -> Option<&Integer> {
    if let Data::Integer(i) = data {
        Some(i)
    } else {
        None
    }
}

pub fn un_bytes(data: &Data) -> Option<&[u8]> {
    if let Data::Bytes(b) = data {
        Some(b)
    } else {
        None
    }
}

pub fn equals(data1: &Data, data2: &Data) -> bool {
    data1 == data2
}

pub fn mk_pair<'a>(first: &'a Data, second: &'a Data) -> (&'a Data, &'a Data) {
    (first, second)
}

pub fn mk_nil(_: ()) -> List<'static> {
    List::Data(&[])
}

pub fn mk_nil_pair(_: ()) -> List<'static> {
    List::PairData(&[])
}

pub fn serialize(data: &Data) -> Vec<u8> {
    tinycbor::to_vec(&data)
}
