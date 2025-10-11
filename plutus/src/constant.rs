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
    Pair(Box<(Constant, Constant)>),
    Data(Data),
    BLSG1Element(Box<bls12_381::G1Affine>),
    BLSG2Element(Box<bls12_381::G2Affine>),
}

impl FromStr for Constant {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ty, constant) = lex::constant_type(s).ok_or(())?;
        let (constant, "") = from_split(ty, constant, false)? else {
            return Err(());
        };

        Ok(constant)
    }
}

fn from_split<'a>(ty: &str, konst: &'a str, split_comma: bool) -> Result<(Constant, &'a str), ()> {
    let (ty, ty_rest) = lex::word(ty);

    let (konst_word, mut konst_rest) = if split_comma {
        konst
            .split_once(',')
            .map(|(a, b)| (a.trim_end(), b.trim_start()))
            .unwrap_or((konst, ""))
    } else {
        (konst, "")
    };
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
            let konst_chars = konst.strip_prefix('"').ok_or(())?.chars();
            let mut inner = String::new();
            let mut escape = false;
            for c in konst_chars {
                if escape {
                    match c {
                        'n' => inner.push('\n'),
                        'r' => inner.push('\r'),
                        't' => inner.push('\t'),
                        '\\' => inner.push('\\'),
                        '"' => inner.push('"'),
                        _ => return Err(()),
                    }
                    escape = false;
                } else if c == '\\' {
                    escape = true;
                } else if c == '"' {
                    break;
                } else {
                    inner.push(c);
                }
            }

            Constant::String(inner.into_boxed_str())
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
            let (data_str, rest) = lex::group(konst).ok_or(())?;
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
        "list" => {
            let mut items = Vec::new();
            let (mut items_str, rest) = lex::list(konst).ok_or(())?;
            while !items_str.is_empty() {
                let (item, rest) = from_split(ty_rest, items_str, true)?;
                items_str = rest;
                items.push(item);
            }
            konst_rest = rest;
            Constant::List(items.into_boxed_slice())
        }
        "pair" => {
            let (first_ty, rest) = lex::constant_type(ty_rest).ok_or(())?;
            let (second_ty, rest) = lex::constant_type(rest).ok_or(())?;
            if !rest.is_empty() {
                return Err(());
            }

            let (konst_str, rest) = lex::group(konst).ok_or(())?;
            konst_rest = rest;
            let (first, rest) = from_split(first_ty, konst_str, true)?;
            let rest = rest.strip_prefix(',').map(|s| s.trim_start()).ok_or(())?;
            let (second, rest) = from_split(second_ty, rest, false)?;
            if !rest.is_empty() {
                return Err(());
            }

            Constant::Pair(Box::new((first, second)))
        }
        _ => return Err(()),
    };

    if !ty_rest.is_empty() {
        return Err(());
    }

    Ok((constant, konst_rest))
}
