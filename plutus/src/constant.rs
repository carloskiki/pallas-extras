use std::str::FromStr;

use crate::{
    data::{self, Data},
    lex,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Integer(rug::Integer),
    Bytes(Vec<u8>),
    String(String),
    Unit,
    Boolean(bool),
    List(List),
    Array(Array),
    Pair(Box<(Constant, Constant)>),
    Data(Data),
    BLSG1Element(Box<blstrs::G1Projective>),
    BLSG2Element(Box<blstrs::G2Projective>),
    MillerLoopResult(Box<blstrs::MillerLoopResult>),
}

impl Constant {
    fn default_for_type(ty: &str) -> Option<Self> {
        let (main_ty, mut rest) = lex::word(ty);

        let ret = match main_ty {
            "integer" => Constant::Integer(Default::default()),
            "bytestring" => Constant::Bytes(Default::default()),
            "string" => Constant::String(Default::default()),
            "bool" => Constant::Boolean(Default::default()),
            "unit" => Constant::Unit,
            "data" => Constant::Data(Default::default()),
            "bls12_381_G1_element" => Constant::BLSG1Element(Box::new(
                <blstrs::G1Projective as k256::elliptic_curve::Group>::identity(),
            )),
            "bls12_381_G2_element" => Constant::BLSG2Element(Box::new(
                <blstrs::G2Projective as k256::elliptic_curve::Group>::identity(),
            )),
            "list" | "array" => {
                let (element_ty, new_rest) = lex::constant_type(rest)?;
                rest = new_rest;

                let element_const = Constant::default_for_type(element_ty)?;
                if main_ty == "array" {
                    Constant::Array(Array::empty(element_const))
                } else {
                    Constant::List(List::empty(element_const))
                }
            }
            "pair" => {
                let (first_ty, new_rest) = lex::constant_type(rest)?;
                let (second_ty, new_rest) = lex::constant_type(new_rest)?;
                rest = new_rest;

                let first_const = Constant::default_for_type(first_ty)?;
                let second_const = Constant::default_for_type(second_ty)?;
                Constant::Pair(Box::new((first_const, second_const)))
            }

            _ => return None,
        };

        if !rest.is_empty() { None } else { Some(ret) }
    }

    pub fn default_from(&self) -> Self {
        match self {
            Constant::Integer(_) => Constant::Integer(Default::default()),
            Constant::Bytes(_) => Constant::Bytes(Default::default()),
            Constant::String(_) => Constant::String(Default::default()),
            Constant::Unit => Constant::Unit,
            Constant::Boolean(_) => Constant::Boolean(Default::default()),
            Constant::List(list) => Constant::List(match &list.elements {
                Ok(elems) => List {
                    elements: Err(Box::new(elems[0].default_from())),
                },
                Err(_) => list.clone(),
            }),
            Constant::Array(arr) => Constant::Array(match &arr.elements {
                Ok(elems) => Array {
                    elements: Err(Box::new(elems[0].default_from())),
                },
                Err(_) => arr.clone(),
            }),
            Constant::Pair(p) => Constant::Pair(Box::new((p.0.default_from(), p.1.default_from()))),
            Constant::Data(_) => Constant::Data(Default::default()),
            Constant::BLSG1Element(_) => Constant::BLSG1Element(Box::new(
                <blstrs::G1Projective as k256::elliptic_curve::Group>::identity(),
            )),
            Constant::BLSG2Element(_) => Constant::BLSG2Element(Box::new(
                <blstrs::G2Projective as k256::elliptic_curve::Group>::identity(),
            )),
            Constant::MillerLoopResult(_) => Constant::MillerLoopResult(Box::default()),
        }
    }
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
    let (ty_start, ty_rest) = lex::word(ty);

    let (konst_word, mut konst_rest) = konst
        .find(',')
        .map(|pos| (konst[..pos].trim_end(), &konst[pos..]))
        .unwrap_or((konst, ""));
    let constant = match ty_start {
        "integer" => {
            let int = rug::Integer::from_str_radix(konst_word, 10).map_err(|_| ())?;
            Constant::Integer(int)
        }
        "bytestring" => {
            let hex = konst_word.strip_prefix("#").ok_or(())?;
            let bytes = const_hex::decode(hex).map_err(|_| ())?;
            Constant::Bytes(bytes)
        }
        "string" => {
            let (string, rest) = lex::string(konst).ok_or(())?;
            konst_rest = rest;

            Constant::String(string)
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
            Constant::Unit
        }
        "data" => {
            // FIXME: https://github.com/IntersectMBO/plutus/issues/7383
            // let (data_str, rest) = lex::group::<b'(', b')'>(konst).ok_or(())?;
            let (data, rest) = if konst.starts_with('(') {
                let (data_str, rest) = lex::group::<b'(', b')'>(konst).ok_or(())?;
                let Some((data, "")) = data::parse_data(data_str) else {
                    return Err(());
                };
                (data, rest)
            } else {
                data::parse_data(konst).ok_or(())?
            };
            konst_rest = rest;
            Constant::Data(data)
        }
        "bls12_381_G1_element" => {
            let hex = konst_word.strip_prefix("0x").ok_or(())?;
            let bytes = const_hex::decode(hex).map_err(|_| ())?;
            Constant::BLSG1Element(Box::new(
                blstrs::G1Affine::from_compressed(&bytes.try_into().map_err(|_| ())?)
                    .into_option()
                    .ok_or(())?
                    .into(),
            ))
        }
        "bls12_381_G2_element" => {
            let hex = konst_word.strip_prefix("0x").ok_or(())?;
            let bytes = const_hex::decode(hex).map_err(|_| ())?;
            Constant::BLSG2Element(Box::new(
                blstrs::G2Affine::from_compressed(&bytes.try_into().map_err(|_| ())?)
                    .into_option()
                    .ok_or(())?
                    .into(),
            ))
        }
        "list" | "array" => {
            let Some((list_ty, "")) = lex::constant_type(ty_rest) else {
                return Err(());
            };
            let mut items = Vec::new();
            let (mut items_str, rest) = lex::group::<b'[', b']'>(konst).ok_or(())?;
            while !items_str.is_empty() {
                let (item, mut list_rest) = from_split(list_ty, items_str)?;
                if let Some(rest) = list_rest.strip_prefix(',') {
                    list_rest = rest.trim_start();
                } else if !list_rest.is_empty() {
                    return Err(());
                }
                items_str = list_rest;
                items.push(item);
            }
            konst_rest = rest;
            if items.is_empty() {
                Constant::default_for_type(ty).ok_or(())?
            } else if ty_start == "array" {
                Constant::Array(Array {
                    elements: Ok(items.into_boxed_slice()),
                })
            } else {
                items.reverse();
                Constant::List(List {
                    elements: Ok(items),
                })
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

impl From<rug::Integer> for Constant {
    fn from(value: rug::Integer) -> Self {
        Constant::Integer(value)
    }
}

impl From<Vec<u8>> for Constant {
    fn from(value: Vec<u8>) -> Self {
        Constant::Bytes(value)
    }
}

impl From<String> for Constant {
    fn from(value: String) -> Self {
        Constant::String(value)
    }
}

impl From<bool> for Constant {
    fn from(value: bool) -> Self {
        Constant::Boolean(value)
    }
}

impl From<Data> for Constant {
    fn from(value: Data) -> Self {
        Constant::Data(value)
    }
}

impl From<blstrs::G1Projective> for Constant {
    fn from(value: blstrs::G1Projective) -> Self {
        Constant::BLSG1Element(Box::new(value))
    }
}

impl From<blstrs::G2Projective> for Constant {
    fn from(value: blstrs::G2Projective) -> Self {
        Constant::BLSG2Element(Box::new(value))
    }
}

impl From<blstrs::MillerLoopResult> for Constant {
    fn from(value: blstrs::MillerLoopResult) -> Self {
        Constant::MillerLoopResult(Box::new(value))
    }
}

impl From<()> for Constant {
    fn from(_: ()) -> Self {
        Constant::Unit
    }
}

impl From<List> for Constant {
    fn from(value: List) -> Self {
        Constant::List(value)
    }
}

impl<T: Into<Constant> + Default> From<Vec<T>> for Constant {
    fn from(value: Vec<T>) -> Self {
        Constant::List(if value.is_empty() {
            List::empty(T::default().into())
        } else {
            let mut elements: Vec<Constant> = value.into_iter().map(Into::into).collect();
            elements.reverse();
            List {
                elements: Ok(elements),
            }
        })
    }
}

impl From<Array> for Constant {
    fn from(value: Array) -> Self {
        Constant::Array(value)
    }
}

impl<T1: Into<Constant>, T2: Into<Constant>> From<(T1, T2)> for Constant {
    fn from(value: (T1, T2)) -> Self {
        Constant::Pair(Box::new((value.0.into(), value.1.into())))
    }
}

impl TryFrom<Constant> for rug::Integer {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::Integer(int) = value {
            Ok(int)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant> for Vec<u8> {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::Bytes(bytes) = value {
            Ok(bytes)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant> for String {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::String(string) = value {
            Ok(string)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant> for bool {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::Boolean(b) = value {
            Ok(b)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant> for Data {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::Data(data) = value {
            Ok(data)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant> for blstrs::G1Projective {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::BLSG1Element(p) = value {
            Ok(*p)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant> for blstrs::G2Projective {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::BLSG2Element(p) = value {
            Ok(*p)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant> for blstrs::MillerLoopResult {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::MillerLoopResult(r) = value {
            Ok(*r)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant> for () {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::Unit = value {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<T: TryFrom<Constant>> TryFrom<Constant> for Vec<T> {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        match value {
            Constant::List(List { elements: Err(_) }) => Ok(Vec::new()),
            Constant::List(List {
                elements: Ok(elems),
            }) => elems
                .into_iter()
                .rev()
                .map(T::try_from)
                .collect::<Result<_, _>>()
                .map_err(|_| ()),

            _ => Err(()),
        }
    }
}

impl TryFrom<Constant> for List {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::List(list) = value {
            Ok(list)
        } else {
            Err(())
        }
    }
}

impl<T: TryFrom<Constant>> TryFrom<Constant> for Box<[T]> {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        match value {
            Constant::Array(Array { elements: Err(_) }) => Ok(Box::new([])),
            Constant::Array(Array {
                elements: Ok(elems),
            }) => elems
                .into_iter()
                .map(T::try_from)
                .collect::<Result<Box<[_]>, _>>()
                .map_err(|_| ()),

            _ => Err(()),
        }
    }
}

impl TryFrom<Constant> for Array {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::Array(array) = value {
            Ok(array)
        } else {
            Err(())
        }
    }
}

impl<T1: TryFrom<Constant>, T2: TryFrom<Constant>> TryFrom<Constant> for (T1, T2) {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::Pair(boxed) = value {
            let first = T1::try_from(boxed.0).map_err(|_| ())?;
            let second = T2::try_from(boxed.1).map_err(|_| ())?;
            Ok((first, second))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct List {
    /// Elements are stored in reverse order for efficient `cons` operation.
    ///
    /// If `Err`, the list is empty but has a known element type.
    ///
    /// INVARIANTS:
    /// - If `Ok`, the list has at least one element.
    /// - All elements in the list have the same type.
    pub elements: Result<Vec<Constant>, Box<Constant>>,
}

impl List {
    pub fn empty(element_type: Constant) -> Self {
        Self {
            elements: Err(Box::new(element_type)),
        }
    }
}

impl<U: Into<Constant> + Default> FromIterator<U> for List {
    fn from_iter<T: IntoIterator<Item = U>>(iter: T) -> Self {
        let mut elements: Vec<Constant> = iter.into_iter().map(Into::into).collect();
        if elements.is_empty() {
            Self::empty(U::default().into())
        } else {
            elements.reverse();
            Self {
                elements: Ok(elements),
            }
        }
    }
}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        match (&self.elements, &other.elements) {
            (Ok(a), Ok(b)) => a == b,
            (Err(a), Err(b)) => std::mem::discriminant(&**a) == std::mem::discriminant(b),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    /// INVARIANTS:
    /// - If `Ok`, the array has at least one element.
    /// - All elements in the array have the same type.
    pub elements: Result<Box<[Constant]>, Box<Constant>>,
}

impl Array {
    pub fn empty(element_type: Constant) -> Self {
        Self {
            elements: Err(Box::new(element_type)),
        }
    }
}

impl<U: Into<Constant> + Default> FromIterator<U> for Array {
    fn from_iter<T: IntoIterator<Item = U>>(iter: T) -> Self {
        let elements: Vec<Constant> = iter.into_iter().map(Into::into).collect();
        if elements.is_empty() {
            Self::empty(U::default().into())
        } else {
            Self {
                elements: Ok(elements.into_boxed_slice()),
            }
        }
    }
}

impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        match (&self.elements, &other.elements) {
            (Ok(a), Ok(b)) => a == b,
            (Err(a), Err(b)) => std::mem::discriminant(&**a) == std::mem::discriminant(b),
            _ => false,
        }
    }
}
