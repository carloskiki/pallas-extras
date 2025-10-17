use crate::data::{Construct, Data};

pub fn choose(data: &Data, construct: u32, map: u32, list: u32, integer: u32, bytes: u32) -> u32 {
    match data {
        Data::Construct(_) => construct,
        Data::Map(_) => map,
        Data::List(_) => list,
        Data::Integer(_) => integer,
        Data::Bytes(_) => bytes,
    }
}

pub fn construct(tag: &rug::Integer, fields: Vec<Data>) -> Data {
    Data::Construct(Construct {
        // We wrap here because this case is quite degenerate.
        // Although the spec strongly suggests using integers that fit in 64 bits,
        // this is not forbidden, although deserialization will fail.
        tag: tag.to_u64_wrapping(),
        value: fields.into_boxed_slice(),
    })
}

pub fn map(pairs: Vec<(Data, Data)>) -> Data {
    Data::Map(pairs.into_boxed_slice())
}

pub fn list(elements: Vec<Data>) -> Data {
    Data::List(elements.into_boxed_slice())
}

pub fn integer(i: rug::Integer) -> Data {
    Data::Integer(i)
}

pub fn bytes(b: Vec<u8>) -> Data {
    Data::Bytes(b.into_boxed_slice())
}

pub fn un_construct(data: Data) -> Option<(rug::Integer, Vec<Data>)> {
    if let Data::Construct(Construct { tag, value }) = data {
        Some((rug::Integer::from(tag), value.into_vec()))
    } else {
        None
    }
}

pub fn un_map(data: Data) -> Option<Vec<(Data, Data)>> {
    if let Data::Map(pairs) = data {
        Some(pairs.into_vec())
    } else {
        None
    }
}

pub fn un_list(data: Data) -> Option<Vec<Data>> {
    if let Data::List(elements) = data {
        Some(elements.into_vec())
    } else {
        None
    }
}

pub fn un_integer(data: Data) -> Option<rug::Integer> {
    if let Data::Integer(i) = data {
        Some(i)
    } else {
        None
    }
}

pub fn un_bytes(data: Data) -> Option<Vec<u8>> {
    if let Data::Bytes(b) = data {
        Some(b.into_vec())
    } else {
        None
    }
}

pub fn equals(data1: &Data, data2: &Data) -> bool {
    data1 == data2
}

pub fn mk_pair(first: Data, second: Data) -> (Data, Data) {
    (first, second)
}

pub fn mk_nil() -> Vec<Data> {
    Vec::new()
}

pub fn mk_nil_pair() -> Vec<(Data, Data)> {
    Vec::new()
}

pub fn serialize(data: &Data) -> Vec<u8> {
    minicbor::to_vec(data).expect("serialization should not fail")
}
