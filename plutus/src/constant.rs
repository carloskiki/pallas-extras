use std::str::FromStr;

use crate::{data::Data, lex};

pub enum Constant {
    Integer(rug::Integer),
    Bytes(Box<[u8]>),
    String(Box<str>),
    Uint,
    Boolean(bool),
    List(Box<[Constant]>),
    Pair(Box<(Constant, Constant)>),
    Data(Data),
    BLSG1Element(Box<bls12_381::G1Affine>),
    BLSG2Element(Box<bls12_381::G2Affine>),
}

impl FromStr for Constant {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ty, constant) = lex::constant_type(s).ok_or(())?;
        let (constant, "") = from_split(ty, constant)? else {
            return Err(());
        };
        
        Ok(constant)
    }
}

fn from_split<'a>(ty: &str, konst: &'a str) -> Result<(Constant, &'a str), ()> {
    let (ty, ty_rest) = lex::word(ty);

    let mut allowed_type_rest = false;
    let (konst_str, mut konst_rest) = konst
        .split_once(char::is_whitespace)
        .map(|(a, b)| (a, b.trim_start()))
        .unwrap_or((konst, ""));
    let constant = match ty {
        "integer" => {
            let int = rug::Integer::from_str_radix(konst_str, 10).map_err(|_| ())?;
            Constant::Integer(int)
        }
        "bytestring" => {
            let hex = konst_str.strip_prefix("#").ok_or(())?;
            let bytes = const_hex::decode(hex).map_err(|_| ())?;
            Constant::Bytes(bytes.into_boxed_slice())
        }
        "string" => {
            if !konst_str.starts_with('"') || !konst_str.ends_with('"') {
                return Err(());
            }
            let inner = &konst_str[1..konst_str.len() - 1];
            Constant::String(inner.to_owned().into_boxed_str())
        }

        "bool" => match konst_str {
            "True" => Constant::Boolean(true),
            "False" => Constant::Boolean(false),
            _ => return Err(()),
        },
        "unit" => {
            if konst_str != "()" {
                return Err(());
            }
            Constant::Uint
        }
        "data" => {
            let (data_str, rest) = lex::group(konst).ok_or(())?;
            konst_rest = rest;
            let data = Data::from_str(data_str)?;
            Constant::Data(data)
        }
        "bls12_381_G1_element" => {
            let hex = konst_str.strip_prefix("0x").ok_or(())?;
            let bytes = const_hex::decode(hex).map_err(|_| ())?;
            Constant::BLSG1Element(Box::new(
                bls12_381::G1Affine::from_compressed(&bytes.try_into().map_err(|_| ())?)
                    .into_option()
                    .ok_or(())?,
            ))
        }
        "bls12_381_G2_element" => {
            let hex = konst_str.strip_prefix("0x").ok_or(())?;
            let bytes = const_hex::decode(hex).map_err(|_| ())?;
            Constant::BLSG2Element(Box::new(
                bls12_381::G2Affine::from_compressed(&bytes.try_into().map_err(|_| ())?)
                    .into_option()
                    .ok_or(())?,
            ))
        }
        "list" => {
            allowed_type_rest = true;
            let mut items = Vec::new();
            let items_str = konst
                .strip_prefix('[')
                .and_then(|s| s.strip_suffix(']'))
                .ok_or(())?
                .trim();
            while !items_str.is_empty() {
                // TODO: here type rest
                let (item, rest) = from_split(ty_rest, items_str)?;
                items.push(item);
                konst_rest = rest;
            }
            Constant::List(items.into_boxed_slice())
        }
        "pair" => {
            allowed_type_rest = true;
            let (first_ty, rest) = lex::constant_type(ty_rest).ok_or(())?;
            let (second_ty, rest) = lex::constant_type(rest).ok_or(())?;
            if !rest.is_empty() {
                return Err(());
            }

            let (first, rest) = from_split(first_ty, konst_str)?;
            let (second, rest) = from_split(second_ty, rest)?;
            konst_rest = rest;

            Constant::Pair(Box::new((first, second)))
        }
        _ => return Err(()),
    };

    if !ty_rest.is_empty() && !allowed_type_rest {
        return Err(());
    }

    Ok((constant, konst_rest))
}
