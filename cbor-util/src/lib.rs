pub mod boxed_slice;
pub mod list_as_map;
pub mod set;
pub mod bounded_bytes;
pub mod bool_as_u8;
pub mod signature;
pub mod cbor_encoded;
pub mod url;
pub mod tinycbor;

use minicbor::{decode::{BytesIter, StrIter}, Decoder};


pub fn bytes_iter_collect(iter: BytesIter<'_, '_>) -> Result<Box<[u8]>, minicbor::decode::Error> {
    let mut bytes = Vec::with_capacity(iter.size_hint().0);
    for chunk in iter {
        bytes.extend_from_slice(chunk?);
    }
    Ok(bytes.into_boxed_slice())
}

pub fn str_iter_collect(iter: StrIter<'_, '_>) -> Result<Box<str>, minicbor::decode::Error> {
    let mut string = String::with_capacity(iter.size_hint().0);
    for chunk in iter {
        string.push_str(chunk?);
    }
    Ok(string.into_boxed_str())
}

pub fn array_decode<'a, T, F: FnOnce(&mut Decoder<'a>) -> Result<T, minicbor::decode::Error>>(
    len: u64,
    f: F,
    d: &mut Decoder<'a>,
) -> Result<T, minicbor::decode::Error> {
    let arr_len = d.array()?;
    if arr_len.is_some_and(|l| l != len) {
        return Err(minicbor::decode::Error::message(format!(
            "expected array of length {}",
            len
        )));
    }
    let ret = f(d)?;

    if arr_len.is_none() {
        if d.datatype()? != minicbor::data::Type::Break {
            return Err(minicbor::decode::Error::message(format!(
                "expected array of length {}",
                len
            )));
        }
        d.skip()?;
    }

    Ok(ret)
}
