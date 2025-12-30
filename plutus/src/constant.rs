//! A plutus constant value.
//!
//! Defined in [the plutus specification][spec] section 4.3.
//!
//! [spec]: https://plutus.cardano.intersectmbo.org/resources/plutus-core-spec.pdf

use std::str::FromStr;

use bwst::{g1, g2, group::GroupEncoding};

use crate::{
    data::{self, Data},
    lex,
};

#[derive(Debug, Clone, PartialEq)]
/// A plutus list constant which stores its type if it is empty.
pub struct List {
    /// Elements are stored in reverse order for efficient `cons` operation.
    ///
    /// INVARIANTS:
    /// - If `Ok`, the list has at least one element.
    /// - All elements in the list have the same type.
    pub elements: Result<Vec<Constant>, Type>,
}

impl List {
    /// Create an empty list with the given element type.
    pub fn empty(element_type: Type) -> Self {
        Self {
            elements: Err(element_type),
        }
    }

    /// Iterate over the elements of the list in order.
    pub fn iter(&self) -> impl Iterator<Item = &Constant> {
        match &self.elements {
            Ok(elems) => elems.iter().rev(),
            Err(_) => [].iter().rev(),
        }
    }

    /// Create a list from a vector of elements and the element type.
    ///
    /// The all the elements and the type must be consistent. Otherwise, the behavior is
    /// undefined.
    pub fn from_vec_ty(mut elements: Vec<Constant>, element_type: Type) -> Self {
        debug_assert!(elements.iter().all(|e| e.type_of() == element_type),);

        if elements.is_empty() {
            Self::empty(element_type)
        } else {
            elements.reverse();
            Self {
                elements: Ok(elements),
            }
        }
    }
}

impl<U: Into<Constant> + Default> FromIterator<U> for List {
    fn from_iter<T: IntoIterator<Item = U>>(iter: T) -> Self {
        let mut elements: Vec<Constant> = iter.into_iter().map(Into::into).collect();
        if elements.is_empty() {
            Self::empty(U::default().into().type_of())
        } else {
            elements.reverse();
            Self {
                elements: Ok(elements),
            }
        }
    }
}

/// A plutus array constant which stores its type if it is empty.
#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    /// INVARIANTS:
    /// - If `Ok`, the array has at least one element.
    /// - All elements in the array have the same type.
    pub elements: Result<Box<[Constant]>, Type>,
}

impl Array {
    /// Create an empty array with the given element type.
    pub fn empty(element_type: Type) -> Self {
        Self {
            elements: Err(element_type),
        }
    }

    /// Iterate over the elements of the array in order.
    pub fn iter(&self) -> impl Iterator<Item = &Constant> {
        match &self.elements {
            Ok(elems) => elems.iter(),
            Err(_) => [].iter(),
        }
    }

    /// Create an array from a boxed slice of elements and the element type.
    ///
    /// The all the elements and the type must be consistent. Otherwise, the behavior is
    /// undefined.
    pub fn from_boxed_ty(elements: Box<[Constant]>, element_type: Type) -> Self {
        debug_assert!(elements.iter().all(|e| e.type_of() == element_type),);

        if elements.is_empty() {
            Self::empty(element_type)
        } else {
            Self {
                elements: Ok(elements),
            }
        }
    }
}

impl<U: Into<Constant> + Default> FromIterator<U> for Array {
    fn from_iter<T: IntoIterator<Item = U>>(iter: T) -> Self {
        let elements: Vec<Constant> = iter.into_iter().map(Into::into).collect();
        if elements.is_empty() {
            Self::empty(U::default().into().type_of())
        } else {
            Self {
                elements: Ok(elements.into_boxed_slice()),
            }
        }
    }
}

/// A plutus constant.
///
/// Defined in [the plutus specification][spec] section 4.3. Constants can come from different
/// "batches" depending on when they were introduced. Each variant documents which batch it belongs
/// to.
///
/// [spec]: https://plutus.cardano.intersectmbo.org/resources/plutus-core-spec.pdf
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Integer(rug::Integer),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Bytes(Vec<u8>),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    String(String),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Unit,
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Boolean(bool),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    List(List),
    /// Introduced in batch 6 (specification section 4.3.6.1).
    Array(Array),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Pair(Box<(Constant, Constant)>),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Data(Data),
    /// Introduced in batch 4 (specification section 4.3.4.2).
    BLSG1Element(Box<g1::Projective>),
    /// Introduced in batch 4 (specification section 4.3.4.2).
    BLSG2Element(Box<g2::Projective>),
    /// Introduced in batch 4 (specification section 4.3.4.2).
    MillerLoopResult(Box<bwst::miller_loop::Result>),
}

impl Constant {
    pub fn type_of(&self) -> Type {
        match self {
            Constant::Integer(_) => Type::Integer,
            Constant::Bytes(_) => Type::Bytes,
            Constant::String(_) => Type::String,
            Constant::Unit => Type::Unit,
            Constant::Boolean(_) => Type::Boolean,
            Constant::List(list) => {
                let element_type = match &list.elements {
                    Ok(elems) => elems[0].type_of(),
                    Err(elem_type) => elem_type.clone(),
                };
                Type::List(Box::new(element_type))
            }
            Constant::Array(array) => {
                let element_type = match &array.elements {
                    Ok(elems) => elems[0].type_of(),
                    Err(elem_type) => elem_type.clone(),
                };
                Type::Array(Box::new(element_type))
            }
            Constant::Pair(boxed) => {
                let first_type = boxed.0.type_of();
                let second_type = boxed.1.type_of();
                Type::Pair(Box::new((first_type, second_type)))
            }
            Constant::Data(_) => Type::Data,
            Constant::BLSG1Element(_) => Type::BLSG1Element,
            Constant::BLSG2Element(_) => Type::BLSG2Element,
            Constant::MillerLoopResult(_) => Type::MillerLoopResult,
        }
    }
}

/// The type of a plutus constant, without its value.
///
/// This is used for type annotations on lists and arrays, and helps when parsing constants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Integer,
    Bytes,
    String,
    Unit,
    Boolean,
    List(Box<Type>),
    Pair(Box<(Type, Type)>),
    Data,
    BLSG1Element,
    BLSG2Element,
    MillerLoopResult,
    Array(Box<Type>),
}

impl FromStr for Type {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (main_ty, mut rest) = lex::word(s);

        let ret = match main_ty {
            "integer" => Type::Integer,
            "bytestring" => Type::Bytes,
            "string" => Type::String,
            "bool" => Type::Boolean,
            "unit" => Type::Unit,
            "data" => Type::Data,
            "bls12_381_G1_element" => Type::BLSG1Element,
            "bls12_381_G2_element" => Type::BLSG2Element,
            "list" | "array" => {
                let (element_ty, new_rest) = lex::constant_type(rest).ok_or(())?;
                rest = new_rest;

                let element_const = Type::from_str(element_ty)?;
                if main_ty == "array" {
                    Type::Array(Box::new(element_const))
                } else {
                    Type::List(Box::new(element_const))
                }
            }
            "pair" => {
                let (first_ty, new_rest) = lex::constant_type(rest).ok_or(())?;
                let (second_ty, new_rest) = lex::constant_type(new_rest).ok_or(())?;
                rest = new_rest;

                let first_const = Type::from_str(first_ty)?;
                let second_const = Type::from_str(second_ty)?;
                Type::Pair(Box::new((first_const, second_const)))
            }

            _ => return Err(()),
        };

        if !rest.is_empty() { Err(()) } else { Ok(ret) }
    }
}

impl FromStr for Constant {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ty, constant) = lex::constant_type(s).ok_or(ParseError::UnknownType)?;
        let (constant, "") = from_split(ty, constant)? else {
            return Err(ParseError::TrailingContent);
        };

        Ok(constant)
    }
}

fn from_split<'a>(ty: &str, konst: &'a str) -> Result<(Constant, &'a str), ParseError> {
    let (ty_start, ty_rest) = lex::word(ty);

    let (konst_word, mut konst_rest) = konst
        .find(',')
        .map(|pos| (konst[..pos].trim_end(), &konst[pos..]))
        .unwrap_or((konst, ""));
    let constant = match ty_start {
        "integer" => {
            let int =
                rug::Integer::from_str_radix(konst_word, 10).map_err(|_| ParseError::Integer)?;
            Constant::Integer(int)
        }
        "bytestring" => {
            let hex = konst_word.strip_prefix("#").ok_or(ParseError::Bytestring)?;
            let bytes = const_hex::decode(hex).map_err(|_| ParseError::Bytestring)?;
            Constant::Bytes(bytes)
        }
        "string" => {
            let (string, rest) = lex::string(konst).ok_or(ParseError::String)?;
            konst_rest = rest;

            Constant::String(string)
        }

        "bool" => match konst_word {
            "True" => Constant::Boolean(true),
            "False" => Constant::Boolean(false),
            _ => return Err(ParseError::Boolean),
        },
        "unit" => {
            if konst_word != "()" {
                return Err(ParseError::Unit);
            }
            Constant::Unit
        }
        "data" => {
            // FIXME: https://github.com/IntersectMBO/plutus/issues/7383
            // let (data_str, rest) = lex::group::<b'(', b')'>(konst).ok_or(())?;
            let (data, rest) = if konst.starts_with('(') {
                let (data_str, rest) = lex::group::<b'(', b')'>(konst).ok_or(ParseError::Data)?;
                let Some((data, "")) = data::parse_data(data_str) else {
                    return Err(ParseError::Data);
                };
                (data, rest)
            } else {
                data::parse_data(konst).ok_or(ParseError::Data)?
            };
            konst_rest = rest;
            Constant::Data(data)
        }
        "bls12_381_G1_element" => {
            let hex = konst_word
                .strip_prefix("0x")
                .ok_or(ParseError::BLSG1Element)?;
            let bytes = const_hex::decode(hex).map_err(|_| ParseError::BLSG1Element)?;
            Constant::BLSG1Element(Box::new(
                g1::Projective::from_bytes(&g1::Compressed(
                    bytes.try_into().map_err(|_| ParseError::BLSG1Element)?,
                ))
                .into_option()
                .ok_or(ParseError::BLSG1Element)?
            ))
        }
        "bls12_381_G2_element" => {
            let hex = konst_word
                .strip_prefix("0x")
                .ok_or(ParseError::BLSG2Element)?;
            let bytes = const_hex::decode(hex).map_err(|_| ParseError::BLSG2Element)?;
            Constant::BLSG2Element(Box::new(
                g2::Projective::from_bytes(&g2::Compressed(
                    bytes.try_into().map_err(|_| ParseError::BLSG2Element)?,
                ))
                .into_option()
                .ok_or(ParseError::BLSG2Element)?
            ))
        }
        "list" | "array" => {
            let Some((list_ty, "")) = lex::constant_type(ty_rest) else {
                return Err(ParseError::List);
            };
            let mut items = Vec::new();
            let (mut items_str, rest) = lex::group::<b'[', b']'>(konst).ok_or(ParseError::List)?;
            while !items_str.is_empty() {
                let (item, mut list_rest) = from_split(list_ty, items_str)?;
                if let Some(rest) = list_rest.strip_prefix(',') {
                    list_rest = rest.trim_start();
                } else if !list_rest.is_empty() {
                    return Err(ParseError::List);
                }
                items_str = list_rest;
                items.push(item);
            }
            konst_rest = rest;
            let element_type = Type::from_str(list_ty).map_err(|_| {
                if ty_start == "array" {
                    ParseError::Array
                } else {
                    ParseError::List
                }
            })?;
            if ty_start == "array" {
                Constant::Array(Array::from_boxed_ty(items.into_boxed_slice(), element_type))
            } else {
                Constant::List(List::from_vec_ty(items, element_type))
            }
        }
        "pair" => {
            let (first_ty, rest) = lex::constant_type(ty_rest).ok_or(ParseError::Pair)?;
            let (second_ty, rest) = lex::constant_type(rest).ok_or(ParseError::Pair)?;
            if !rest.is_empty() {
                return Err(ParseError::Pair);
            }

            let (konst_str, rest) = lex::group::<b'(', b')'>(konst).ok_or(ParseError::Pair)?;
            konst_rest = rest;
            let (first, rest) = from_split(first_ty, konst_str)?;
            let (second, rest) = from_split(
                second_ty,
                rest.strip_prefix(',').ok_or(ParseError::Pair)?.trim_start(),
            )?;
            if !rest.is_empty() {
                return Err(ParseError::Pair);
            }

            Constant::Pair(Box::new((first, second)))
        }
        _ => return Err(ParseError::UnknownType),
    };

    Ok((constant, konst_rest))
}

/// An error that can occur when parsing a constant.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum ParseError {
    #[error("unknown constant type")]
    UnknownType,
    #[error("invalid integer format")]
    Integer,
    #[error("invalid bytestring format")]
    Bytestring,
    #[error("invalid string format")]
    String,
    #[error("invalid boolean format")]
    Boolean,
    #[error("invalid unit format")]
    Unit,
    #[error("invalid data format")]
    Data,
    #[error("invalid BLS G1 element format")]
    BLSG1Element,
    #[error("invalid BLS G2 element format")]
    BLSG2Element,
    #[error("invalid list format")]
    List,
    #[error("invalid array format")]
    Array,
    #[error("invalid pair format")]
    Pair,
    #[error("trailing content after constant")]
    TrailingContent,
}

// Implement `From` and `TryFrom` to help with macro-generated builtin type handling.

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

impl From<g1::Projective> for Constant {
    fn from(value: g1::Projective) -> Self {
        Constant::BLSG1Element(Box::new(value))
    }
}

impl From<g2::Projective> for Constant {
    fn from(value: g2::Projective) -> Self {
        Constant::BLSG2Element(Box::new(value))
    }
}

impl From<bwst::miller_loop::Result> for Constant {
    fn from(value: bwst::miller_loop::Result) -> Self {
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
            List::empty(T::default().into().type_of())
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

impl<T: Into<Constant> + Default> From<Box<[T]>> for Constant {
    fn from(value: Box<[T]>) -> Self {
        Constant::Array(if value.is_empty() {
            Array::empty(T::default().into().type_of())
        } else {
            let elements: Box<[Constant]> = value.into_iter().map(Into::into).collect();
            Array {
                elements: Ok(elements),
            }
        })
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

impl TryFrom<Constant> for g1::Projective {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::BLSG1Element(p) = value {
            Ok(*p)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant> for g2::Projective {
    type Error = ();

    fn try_from(value: Constant) -> Result<Self, Self::Error> {
        if let Constant::BLSG2Element(p) = value {
            Ok(*p)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant> for bwst::miller_loop::Result {
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
