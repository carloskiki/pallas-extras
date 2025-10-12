use std::str::FromStr;

use crate::{data::Data, lex};

#[derive(Debug)]
pub enum Constant {
    Integer(rug::Integer),
    Bytes(Box<[u8]>),
    String(Box<str>),
    Uint,
    Boolean(bool),
    List(Box<[Constant]>),
    Array(Box<[Constant]>),
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

    let (konst_word, mut konst_rest) = konst
        .find(',')
        .map(|pos| (konst[..pos].trim_end(), &konst[pos..]))
        .unwrap_or((konst, ""));
    let constant = match ty {
        "integer" => {
            let int = rug::Integer::from_str_radix(konst_word, 10).map_err(|_| ())?;
            Constant::Integer(int)
        }
        "bytestring" => {
            let hex = konst_word.strip_prefix("#").ok_or(())?;
            let bytes = const_hex::decode(hex).map_err(|_| ())?;
            Constant::Bytes(bytes.into_boxed_slice())
        }
        "string" => {
            let (string, rest) = lex::string(konst).ok_or(())?;
            konst_rest = rest;

            Constant::String(string.into_boxed_str())
        }

        "bool" => match konst_word {
            "True" => Constant::Boolean(true),
            "False" => Constant::Boolean(false),
            _ => return Err(()),
        },
        "unit" => {
            if konst_word != "()" {
                return Err(());
            }
            Constant::Uint
        }
        "data" => {
            let (data_str, rest) = lex::group::<b'(', b')'>(konst).ok_or(())?;
            konst_rest = rest;
            let data = Data::from_str(data_str)?;
            Constant::Data(data)
        }
        "bls12_381_G1_element" => {
            let hex = konst_word.strip_prefix("0x").ok_or(())?;
            let bytes = const_hex::decode(hex).map_err(|_| ())?;
            Constant::BLSG1Element(Box::new(
                bls12_381::G1Affine::from_compressed(&bytes.try_into().map_err(|_| ())?)
                    .into_option()
                    .ok_or(())?,
            ))
        }
        "bls12_381_G2_element" => {
            let hex = konst_word.strip_prefix("0x").ok_or(())?;
            let bytes = const_hex::decode(hex).map_err(|_| ())?;
            Constant::BLSG2Element(Box::new(
                bls12_381::G2Affine::from_compressed(&bytes.try_into().map_err(|_| ())?)
                    .into_option()
                    .ok_or(())?,
            ))
        }
        "list" | "array" => {
            let mut items = Vec::new();
            let (mut items_str, rest) = lex::group::<b'[', b']'>(konst).ok_or(())?;
            while !items_str.is_empty() {
                let (item, mut list_rest) = from_split(ty_rest, items_str)?;
                if let Some(rest) = list_rest.strip_prefix(',') {
                    list_rest = rest.trim_start();
                } else if !list_rest.is_empty() {
                    return Err(());
                }
                items_str = list_rest;
                items.push(item);
            }
            konst_rest = rest;
            if ty == "array" {
                Constant::Array(items.into_boxed_slice())
            } else {
                Constant::List(items.into_boxed_slice())
            }
        }
        "pair" => {
            let (first_ty, rest) = lex::constant_type(ty_rest).ok_or(())?;
            let (second_ty, rest) = lex::constant_type(rest).ok_or(())?;
            if !rest.is_empty() {
                return Err(());
            }

            let (konst_str, rest) = lex::group::<b'(', b')'>(konst).ok_or(())?;
            konst_rest = rest;
            let (first, rest) = from_split(first_ty, konst_str)?;
            let (second, rest) =
                from_split(second_ty, rest.strip_prefix(',').ok_or(())?.trim_start())?;
            if !rest.is_empty() {
                return Err(());
            }

            Constant::Pair(Box::new((first, second)))
        }
        _ => return Err(()),
    };

    Ok((constant, konst_rest))
}
