//! A plutus constant value.
//!
//! Defined in [the plutus specification][spec] section 4.3.
//!
//! [spec]: https://plutus.cardano.intersectmbo.org/resources/plutus-core-spec.pdf

use crate::{Construct, Data, lex};
use bwst::{g1, g2, group::GroupEncoding};
use std::str::FromStr;

mod arena;
pub use arena::Arena;

// TODO: This is quite messy, consider refactoring.

pub type Array<'a> = List<'a>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum List<'a> {
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Integer(&'a [rug::Integer]),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Bytes(&'a [&'a [u8]]), // X
    /// Introduced in batch 1 (specification section 4.3.1.1).
    String(&'a [&'a str]), // X
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Unit(&'a [()]), // X
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Boolean(&'a [bool]), // X
    /// Introduced in batch 1 (specification section 4.3.1.1).
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Data(&'a [Data]),
    /// Specialized for faster processing of `UnMapData`.
    PairData(&'a [(Data, Data)]),
    /// Introduced in batch 4 (specification section 4.3.4.2).
    BLSG1Element(&'a [g1::Projective]),
    /// Introduced in batch 4 (specification section 4.3.4.2).
    BLSG2Element(&'a [g2::Projective]),
    /// Introduced in batch 4 (specification section 4.3.4.2).
    MillerLoopResult(&'a [bwst::miller_loop::Result]), // X
    /// Generic list for when the type is nested.
    ///
    /// This includes:
    /// - list (pair _ _)
    /// - list (list _)
    /// - list (array _)
    Generic(Result<&'a [Constant<'a>], Type<'a>>),
}

impl<'a> List<'a> {
    pub fn is_empty(&self) -> bool {
        match self {
            List::Integer(items) => items.is_empty(),
            List::Bytes(items) => items.is_empty(),
            List::String(items) => items.is_empty(),
            List::Unit(items) => items.is_empty(),
            List::Boolean(items) => items.is_empty(),
            List::Data(items) => items.is_empty(),
            List::PairData(items) => items.is_empty(),
            List::BLSG1Element(items) => items.is_empty(),
            List::BLSG2Element(items) => items.is_empty(),
            List::MillerLoopResult(items) => items.is_empty(),
            List::Generic(Ok(items)) => items.is_empty(),
            List::Generic(Err(_)) => true,
        }
    }

    pub fn destructure(&self, arena: &'a Arena) -> Option<(Constant<'a>, List<'a>)> {
        macro_rules! destructure {
            ($variable:ident, $variant:ident) => {
                destructure!($variable, $variant | ($variable))
            };
            ($variable:ident, $variant:ident | $op:tt) => {
                $variable
                    .split_first()
                    .map(|($variable, rest)| (Constant::$variant $op, List::$variant(rest)))
            };
        }

        match self {
            List::Integer(items) => destructure!(items, Integer),
            List::Bytes(items) => destructure!(items, Bytes),
            List::String(items) => destructure!(items, String),
            #[allow(unused_variables)]
            List::Unit(items) => destructure!(items, Unit | {}),
            List::Boolean(items) => destructure!(items, Boolean | (*items)),
            List::Data(datas) => destructure!(datas, Data),
            List::PairData(items) => items.split_first().map(|((k, v), rest)| {
                (
                    Constant::Pair(
                        arena.alloc(Constant::Data(k)),
                        arena.alloc(Constant::Data(v)),
                    ),
                    List::PairData(rest),
                )
            }),
            List::BLSG1Element(projectives) => destructure!(projectives, BLSG1Element),
            List::BLSG2Element(projectives) => destructure!(projectives, BLSG2Element),
            List::MillerLoopResult(items) => destructure!(items, MillerLoopResult),
            List::Generic(Ok(items)) => {
                let (first, rest) = items.split_first().expect("non-empty list");
                Some((
                    *first,
                    List::Generic(if rest.is_empty() {
                        Err(first.type_of(arena))
                    } else {
                        Ok(rest)
                    }),
                ))
            }
            List::Generic(Err(_)) => None,
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constant<'a> {
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Integer(&'a rug::Integer),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Bytes(&'a [u8]),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    String(&'a str),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Unit,
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Boolean(bool),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    List(List<'a>),
    /// Introduced in batch 6 (specification section 4.3.6.1).
    Array(Array<'a>),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Pair(&'a Constant<'a>, &'a Constant<'a>),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Data(&'a Data),
    /// Introduced in batch 4 (specification section 4.3.4.2).
    BLSG1Element(&'a g1::Projective),
    /// Introduced in batch 4 (specification section 4.3.4.2).
    BLSG2Element(&'a g2::Projective),
    /// Introduced in batch 4 (specification section 4.3.4.2).
    MillerLoopResult(&'a bwst::miller_loop::Result),
}

impl<'a> Constant<'a> {
    pub fn type_of(&self, arena: &'a Arena) -> Type<'a> {
        fn list_type_of<'a>(list: &List<'a>, arena: &'a Arena) -> Type<'a> {
            match list {
                List::Integer(_) => Type::Integer,
                List::Bytes(_) => Type::Bytes,
                List::String(_) => Type::String,
                List::Unit(_) => Type::Unit,
                List::Boolean(_) => Type::Boolean,
                List::Data(_) => Type::Data,
                List::PairData(_) => Type::Pair(arena.alloc((Type::Data, Type::Data))),
                List::BLSG1Element(_) => Type::BLSG1Element,
                List::BLSG2Element(_) => Type::BLSG2Element,
                List::MillerLoopResult(_) => Type::MillerLoopResult,
                List::Generic(Err(ty)) => *ty,
                List::Generic(Ok(elements)) => {
                    elements.first().expect("non-empty list").type_of(arena)
                }
            }
        }

        match self {
            Constant::Integer(_) => Type::Integer,
            Constant::Bytes(_) => Type::Bytes,
            Constant::String(_) => Type::String,
            Constant::Unit => Type::Unit,
            Constant::Boolean(_) => Type::Boolean,
            Constant::List(list) => Type::List(arena.alloc(list_type_of(list, arena))),
            Constant::Array(array) => Type::Array(arena.alloc(list_type_of(array, arena))),
            Constant::Pair(first, second) => {
                Type::Pair(arena.alloc((first.type_of(arena), second.type_of(arena))))
            }
            Constant::Data(_) => Type::Data,
            Constant::BLSG1Element(_) => Type::BLSG1Element,
            Constant::BLSG2Element(_) => Type::BLSG2Element,
            Constant::MillerLoopResult(_) => Type::MillerLoopResult,
        }
    }

    pub fn from_str(s: &str, arena: &'a Arena) -> Result<Self, ParseError> {
        let (ty_str, rest) = lex::constant_type(s).ok_or(ParseError::UnknownType)?;
        let (constant, rest) = from_split(ty_str, rest.trim_start(), arena)?;
        if !rest.is_empty() {
            Err(ParseError::TrailingContent)
        } else {
            Ok(constant)
        }
    }
}

/// The type of a plutus constant, without its value.
///
/// This is used for type annotations on lists and arrays, and helps when parsing constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type<'a> {
    Integer,
    Bytes,
    String,
    Unit,
    Boolean,
    List(&'a Type<'a>),
    Pair(&'a (Type<'a>, Type<'a>)),
    Data,
    BLSG1Element,
    BLSG2Element,
    MillerLoopResult,
    Array(&'a Type<'a>),
}

fn type_from_str<'a>(s: &str, arena: &'a Arena) -> Result<Type<'a>, ()> {
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

            let element_const = type_from_str(element_ty, arena)?;
            if main_ty == "array" {
                Type::Array(arena.alloc(element_const))
            } else {
                Type::List(arena.alloc(element_const))
            }
        }
        "pair" => {
            let (first_ty, new_rest) = lex::constant_type(rest).ok_or(())?;
            let (second_ty, new_rest) = lex::constant_type(new_rest).ok_or(())?;
            rest = new_rest;

            let first_const = type_from_str(first_ty, arena)?;
            let second_const = type_from_str(second_ty, arena)?;
            Type::Pair(arena.alloc((first_const, second_const)))
        }

        _ => return Err(()),
    };

    if !rest.is_empty() { Err(()) } else { Ok(ret) }
}

fn from_split<'a, 'b>(
    ty: &str,
    konst: &'b str,
    arena: &'a Arena,
) -> Result<(Constant<'a>, &'b str), ParseError> {
    let (ty_start, ty_rest) = lex::word(ty);

    let (konst_word, mut konst_rest) = konst
        .find(',')
        .map(|pos| (konst[..pos].trim_end(), &konst[pos..]))
        .unwrap_or((konst, ""));
    let constant = match ty_start {
        "integer" => integer(konst_word)
            .ok_or(ParseError::Integer)
            .map(|i| Constant::Integer(arena.integer(i)))?,
        "bytestring" => bytestring(konst_word, arena)
            .ok_or(ParseError::Bytestring)
            .map(Constant::Bytes)?,
        "string" => {
            let (string, rest) = lex::string(konst_word).ok_or(ParseError::String)?;
            konst_rest = rest;
            let string = arena.string(&string);

            Constant::String(string)
        }

        "bool" => bool(konst_word)
            .ok_or(ParseError::Boolean)
            .map(Constant::Boolean)?,
        "unit" => unit(konst_word)
            .ok_or(ParseError::Unit)
            .map(|_| Constant::Unit)?,
        "data" => {
            // FIXME: https://github.com/IntersectMBO/plutus/issues/7383
            // let (data_str, rest) = lex::group::<b'(', b')'>(konst).ok_or(())?;
            let (data, rest) = if konst.starts_with('(') {
                let (data_str, rest) = lex::group::<b'(', b')'>(konst).ok_or(ParseError::Data)?;
                let Some((data, "")) = data(data_str) else {
                    return Err(ParseError::Data);
                };
                (data, rest)
            } else {
                data(konst).ok_or(ParseError::Data)?
            };
            konst_rest = rest;
            Constant::Data(arena.data(data))
        }
        "bls12_381_G1_element" => g1(konst_word)
            .ok_or(ParseError::BLSG1Element)
            .map(|p| Constant::BLSG1Element(arena.alloc(p)))?,
        "bls12_381_G2_element" => g2(konst_word)
            .ok_or(ParseError::BLSG2Element)
            .map(|p| Constant::BLSG2Element(arena.alloc(p)))?,
        "list" | "array" => {
            let err_type = if ty_start == "list" {
                ParseError::List
            } else {
                ParseError::Array
            };
            let Some((element_ty, "")) = lex::constant_type(ty_rest) else {
                return Err(err_type);
            };
            let (konst_str, rest) = lex::group::<b'[', b']'>(konst).ok_or(err_type)?;
            konst_rest = rest;
            let list = list_from_split(element_ty, konst_str, arena)?;
            if ty_start == "list" {
                Constant::List(list.into())
            } else {
                Constant::Array(list.into())
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
            let (first, rest) = from_split(first_ty, konst_str, arena)?;
            let (second, rest) = from_split(
                second_ty,
                rest.strip_prefix(',').ok_or(ParseError::Pair)?.trim_start(),
                arena,
            )?;
            if !rest.is_empty() {
                return Err(ParseError::Pair);
            }

            Constant::Pair(arena.alloc(first), arena.alloc(second))
        }
        _ => return Err(ParseError::UnknownType),
    };

    Ok((constant, konst_rest))
}

fn list_from_split<'a>(
    ty: &str,
    items_str: &str,
    arena: &'a Arena,
) -> Result<List<'a>, ParseError> {
    let (ty_start, ty_rest) = lex::word(ty);
    let words = items_str.split(',').map(str::trim);

    match ty_start {
        "integer" => words
            .map(integer)
            .collect::<Option<Vec<rug::Integer>>>()
            .ok_or(ParseError::Integer)
            .map(|ints| List::Integer(arena.integers(ints))),
        "bytestring" => words
            .map(|w| bytestring(w, arena))
            .collect::<Option<Vec<&'a [u8]>>>()
            .ok_or(ParseError::Bytestring)
            .map(|bytes| List::Bytes(arena.slice_fill(bytes.into_iter()))),
        "string" => list_from_fn(items_str, lex::string)
            .ok_or(ParseError::String)
            .map(|strings| {
                List::String(
                    arena.slice_fill(strings.into_iter().map(|s| arena.string(&s) as &str)),
                )
            }),
        "bool" => words
            .map(bool)
            .collect::<Option<Vec<bool>>>()
            .ok_or(ParseError::Boolean)
            .map(|bools| List::Boolean(arena.slice_fill(bools.into_iter()))),
        "unit" => words
            .map(unit)
            .collect::<Option<Vec<()>>>()
            .ok_or(ParseError::Unit)
            .map(|units| List::Unit(arena.slice_fill(units.into_iter()))),
        "data" => list_from_fn(items_str, data)
            .ok_or(ParseError::Data)
            .map(|data| List::Data(arena.datas(data))),
        "bls12_381_G1_element" => words
            .map(g1)
            .collect::<Option<Vec<g1::Projective>>>()
            .ok_or(ParseError::BLSG1Element)
            .map(|g1s| List::BLSG1Element(arena.slice_fill(g1s.into_iter()))),
        "bls12_381_G2_element" => words
            .map(g2)
            .collect::<Option<Vec<g2::Projective>>>()
            .ok_or(ParseError::BLSG2Element)
            .map(|g2s| List::BLSG2Element(arena.slice_fill(g2s.into_iter()))),
        "list" if lex::constant_type(ty_rest) == Some(("pair data data", "")) => {
            list_from_fn(items_str, |s| {
                let (pair_str, rest) = lex::group::<b'(', b')'>(s)?;
                let (key, r) = data(pair_str)?;
                let Some((value, "")) = data(r.strip_prefix(',')?.trim_start()) else {
                    return None;
                };
                Some(((key, value), rest))
            })
            .ok_or(ParseError::List)
            .map(|pairs| List::PairData(arena.pair_data(pairs)))
        }

        "list" | "array" | "pair" => {
            let mut items = Vec::new();
            let mut items_str = items_str;
            while !items_str.is_empty() {
                let (item, mut list_rest) = from_split(ty, items_str, arena)?;
                if let Some(rest) = list_rest.strip_prefix(',') {
                    list_rest = rest.trim_start();
                } else if !list_rest.is_empty() {
                    return Err(ParseError::List);
                }
                items_str = list_rest;
                items.push(item);
            }
            if items.is_empty() {
                Ok(List::Generic(Err(
                    type_from_str(ty, arena).map_err(|_| ParseError::UnknownType)?
                )))
            } else {
                Ok(List::Generic(Ok(arena.slice_fill(items.into_iter()))))
            }
        }
        _ => return Err(ParseError::UnknownType),
    }
}

fn integer(s: &str) -> Option<rug::Integer> {
    rug::Integer::from_str_radix(s, 10).ok()
}

fn bytestring<'a>(s: &str, arena: &'a Arena) -> Option<&'a [u8]> {
    let hex = s.strip_prefix("#")?;
    let value = const_hex::decode(hex).ok()?;
    Some(arena.slice_fill(value))
}

fn bool(s: &str) -> Option<bool> {
    match s {
        "True" => Some(true),
        "False" => Some(false),
        _ => None,
    }
}

fn unit(s: &str) -> Option<()> {
    (s == "()").then_some(())
}

fn g1(s: &str) -> Option<g1::Projective> {
    let hex = s.strip_prefix("0x")?;
    let bytes = const_hex::decode(hex).ok()?;
    g1::Projective::from_bytes(&g1::Compressed(bytes.try_into().ok()?)).into_option()
}

fn g2(s: &str) -> Option<g2::Projective> {
    let hex = s.strip_prefix("0x")?;
    let bytes = const_hex::decode(hex).ok()?;
    g2::Projective::from_bytes(&g2::Compressed(bytes.try_into().ok()?)).into_option()
}

fn data(s: &str) -> Option<(Data, &str)> {
    let (ty, data_str) = s
        .split_once(char::is_whitespace)
        .map(|(a, b)| (a, b.trim_start()))
        .unwrap_or((s, ""));
    Some(match ty {
        "B" => {
            let data_str = data_str.strip_prefix("#")?;
            let (hex, rest) = data_str
                .find(|c: char| !c.is_ascii_hexdigit())
                .map(|pos| (&data_str[..pos], data_str[pos..].trim_start()))
                .unwrap_or((data_str.trim_end(), ""));
            let bytes = const_hex::decode(hex).ok()?;
            (Data::Bytes(bytes), rest.trim_start())
        }
        "I" => {
            let (int_str, rest) = data_str
                .find(|c: char| !c.is_ascii_digit() && c != '-')
                .map(|pos| (&data_str[..pos], data_str[pos..].trim_start()))
                .unwrap_or((data_str.trim_end(), ""));
            let int = rug::Integer::from_str_radix(int_str, 10).ok()?;
            (Data::Integer(int), rest.trim_start())
        }
        "List" => {
            let (items_str, rest) = lex::group::<b'[', b']'>(data_str)?;
            (Data::List(list_from_fn(items_str, data)?), rest)
        }
        "Map" => {
            let (mut items_str, rest) = lex::group::<b'[', b']'>(data_str)?;
            let mut items = Vec::new();
            while !items_str.is_empty() {
                let (pair, other_pairs) = lex::group::<b'(', b')'>(items_str)?;
                items_str = other_pairs
                    .strip_prefix(',')
                    .map(|s| s.trim_start())
                    .unwrap_or(other_pairs);

                let (key, rest) = data(pair)?;
                let (value, "") = data(rest.strip_prefix(',')?.trim_start())? else {
                    return None;
                };

                items.push((key, value));
            }

            (Data::Map(items), rest)
        }
        "Constr" => {
            let (tag_str, fields) = data_str.split_once(|c: char| !c.is_ascii_digit())?;
            let tag = u64::from_str(tag_str).ok()?;
            let (fields, rest) = lex::group::<b'[', b']'>(fields.trim_start())?;
            let value = list_from_fn(fields, data)?;
            (Data::Construct(Construct { tag, value }), rest)
        }
        _ => return None,
    })
}

fn list_from_fn<T>(
    items_str: &str,
    f: impl for<'a> Fn(&'a str) -> Option<(T, &'a str)>,
) -> Option<Vec<T>> {
    let mut items = Vec::new();
    let mut rest = items_str;
    while !rest.is_empty() {
        let (item, new_rest) = f(rest)?;
        items.push(item);
        if let Some(r) = new_rest.strip_prefix(',') {
            rest = r.trim_start();
        } else if !rest.is_empty() {
            return None;
        }
    }
    Some(items)
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

impl<'a> TryFrom<Constant<'a>> for &'a rug::Integer {
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::Integer(i) = value {
            Ok(i)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant<'_>> for rug::Integer {
    type Error = ();

    fn try_from(value: Constant<'_>) -> Result<Self, Self::Error> {
        if let Constant::Integer(i) = value {
            Ok(i.clone())
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<Constant<'a>> for &'a [u8] {
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::Bytes(b) = value {
            Ok(b)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant<'_>> for Vec<u8> {
    type Error = ();

    fn try_from(value: Constant<'_>) -> Result<Self, Self::Error> {
        if let Constant::Bytes(b) = value {
            Ok(b.to_vec())
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<Constant<'a>> for &'a str {
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::String(s) = value {
            Ok(s)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant<'_>> for String {
    type Error = ();

    fn try_from(value: Constant<'_>) -> Result<Self, Self::Error> {
        if let Constant::String(s) = value {
            Ok(s.to_string())
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant<'_>> for () {
    type Error = ();

    fn try_from(value: Constant<'_>) -> Result<Self, Self::Error> {
        if let Constant::Unit = value {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant<'_>> for bool {
    type Error = ();

    fn try_from(value: Constant<'_>) -> Result<Self, Self::Error> {
        if let Constant::Boolean(b) = value {
            Ok(b)
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<Constant<'a>> for List<'a> {
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::List(l) = value {
            Ok(l)
        } else {
            Err(())
        }
    }
}

// TODO: Impl for Array.

impl<'a, A, B> TryFrom<Constant<'a>> for (A, B)
where
    A: TryFrom<Constant<'a>, Error = ()>,
    B: TryFrom<Constant<'a>, Error = ()>,
{
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::Pair(k, v) = value {
            let k = (*k).try_into()?;
            let v = (*v).try_into()?;
            Ok((k, v))
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<Constant<'a>> for &'a Data {
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::Data(d) = value {
            Ok(d)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant<'_>> for g1::Projective {
    type Error = ();

    fn try_from(value: Constant<'_>) -> Result<Self, Self::Error> {
        if let Constant::BLSG1Element(p) = value {
            Ok(*p)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant<'_>> for g2::Projective {
    type Error = ();

    fn try_from(value: Constant<'_>) -> Result<Self, Self::Error> {
        if let Constant::BLSG2Element(p) = value {
            Ok(*p)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant<'_>> for bwst::miller_loop::Result {
    type Error = ();

    fn try_from(value: Constant<'_>) -> Result<Self, Self::Error> {
        if let Constant::MillerLoopResult(r) = value {
            Ok(*r)
        } else {
            Err(())
        }
    }
}


