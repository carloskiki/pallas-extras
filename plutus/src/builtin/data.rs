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

pub fn construct(tag: u64, fields: Vec<Data>) -> Data {
    Data::Construct(Construct {
        tag,
        value: fields.into_boxed_slice(),
    })
}

pub fn map(pairs: Vec<(Data, Data)>) -> Data {
    Data::Map(pairs.into_boxed_slice())
}

pub fn list(elements: Vec<Data>) -> Data {
    Data::List(elements.into_boxed_slice())
}

pub fn integer(i: Integer) -> Data {
    Data::Integer(i)
}

pub fn bytes(b: Vec<u8>) -> Data {
    Data::Bytes(b.into_boxed_slice())
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

pub fn mk_nil<'a>(_: ()) -> List<'a> {
    List::Data(&[])
}

pub fn mk_nil_pair<'a>(_: ()) -> List<'a> {
    List::PairData(&[])
}

pub fn serialize(data: &Data) -> Vec<u8> {
    // TODO: This should totally be a borrowed bytes no-op, `tinycbor::Memo`
    tinycbor::to_vec(&data)
}
