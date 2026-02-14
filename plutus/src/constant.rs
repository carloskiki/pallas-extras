//! A plutus constant value.
//!
//! Defined in [the plutus specification][spec] section 4.3.
//!
//! [spec]: https://plutus.cardano.intersectmbo.org/resources/plutus-core-spec.pdf

use crate::{Construct, Data, builtin::Output, lex};
use bwst::{g1, g2, group::GroupEncoding};
use mitsein::slice1::Slice1;
use std::str::FromStr;

mod arena;
pub use arena::Arena;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Array<'a>(pub List<'a>);

#[derive(Debug, Copy, Clone)]
pub enum List<'a> {
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Integer(&'a [rug::Integer]),
    /// Introduced in batch 1 (specification section 4.3.1.1).
    Data(&'a [Data]),
    /// Specialized for faster processing of `UnMapData`.
    PairData(&'a [(Data, Data)]),
    /// Introduced in batch 4 (specification section 4.3.4.2).
    BLSG1Element(&'a [g1::Projective]),
    /// Introduced in batch 4 (specification section 4.3.4.2).
    BLSG2Element(&'a [g2::Projective]),
    /// Generic list for when the type is nested.
    ///
    /// This includes:
    /// - list (pair _ _)
    /// - list (list _)
    /// - list (array _)
    Generic(Result<&'a Slice1<Constant<'a>>, &'a Constant<'a>>),
}

impl PartialEq for List<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (List::Integer(a), List::Integer(b)) => a == b,
            (List::Data(a), List::Data(b)) => a == b,
            (List::PairData(a), List::PairData(b)) => a == b,
            (List::BLSG1Element(a), List::BLSG1Element(b)) => a == b,
            (List::BLSG2Element(a), List::BLSG2Element(b)) => a == b,
            (List::Generic(Ok(a)), List::Generic(Ok(b))) => a == b,
            (List::Generic(Err(a)), List::Generic(Err(b))) => a.type_eq(b),
            _ => false,
        }
    }
}

impl<'a> List<'a> {
    pub const INTEGER_TYPE: List<'static> = List::Integer(&[]);
    pub const DATA_TYPE: List<'static> = List::Data(&[]);
    pub const PAIRDATA_TYPE: List<'static> = List::PairData(&[]);

    pub fn type_of(&self) -> &Constant<'a> {
        match self {
            List::Integer(_) => &Constant::INTEGER_TYPE,
            List::Data(_) => &Constant::DATA_TYPE,
            List::PairData(_) => &Constant::PAIRDATA_TYPE,
            List::BLSG1Element(_) => &Constant::BLSG1Element(&g1::Projective::IDENTITY),
            List::BLSG2Element(_) => &Constant::BLSG2Element(&g2::Projective::IDENTITY),
            List::Generic(Ok(slice)) => slice.first(),
            List::Generic(Err(ty)) => ty,
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
    // TODO: handle pairData in decode, encode, etc.
    PairData(&'a (Data, Data)),
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
    pub const INTEGER_TYPE: Constant<'static> = Constant::Integer(&rug::Integer::new());
    pub const BYTES_TYPE: Constant<'static> = Constant::Bytes(&[]);
    pub const STRING_TYPE: Constant<'static> = Constant::String("");
    pub const BOOLEAN_TYPE: Constant<'static> = Constant::Boolean(false);
    pub const UNIT_TYPE: Constant<'static> = Constant::Unit;
    pub const DATA_TYPE: Constant<'static> = Constant::Data(&Data::Integer(rug::Integer::new()));
    pub const PAIRDATA_TYPE: Constant<'static> = Constant::PairData(&(
        Data::Integer(rug::Integer::new()),
        Data::Integer(rug::Integer::new()),
    ));
    pub const BLSG1_TYPE: Constant<'static> = Constant::BLSG1Element(&g1::Projective::IDENTITY);
    pub const BLSG2_TYPE: Constant<'static> = Constant::BLSG2Element(&g2::Projective::IDENTITY);

    pub fn from_str(s: &str, arena: &'a Arena) -> Result<Self, ParseError> {
        let (ty_str, rest) = lex::constant_type(s).ok_or(ParseError::UnknownType)?;
        let (constant, rest) = from_split(ty_str, rest.trim_start(), arena)?;
        if !rest.is_empty() {
            Err(ParseError::TrailingContent)
        } else {
            Ok(constant)
        }
    }

    pub fn type_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Constant::Integer(_), Constant::Integer(_))
            | (Constant::Bytes(_), Constant::Bytes(_))
            | (Constant::String(_), Constant::String(_))
            | (Constant::Unit, Constant::Unit)
            | (Constant::PairData(_), Constant::PairData(_))
            | (Constant::Data(_), Constant::Data(_))
            | (Constant::BLSG1Element(_), Constant::BLSG1Element(_))
            | (Constant::BLSG2Element(_), Constant::BLSG2Element(_))
            | (Constant::MillerLoopResult(_), Constant::MillerLoopResult(_))
            | (Constant::Boolean(_), Constant::Boolean(_)) => true,
            (Constant::List(l0), Constant::List(l1))
            | (Constant::Array(Array(l0)), Constant::Array(Array(l1))) => match (l0, l1) {
                (List::Integer(_), List::Integer(_))
                | (List::Data(_), List::Data(_))
                | (List::PairData(_), List::PairData(_))
                | (List::BLSG1Element(_), List::BLSG1Element(_))
                | (List::BLSG2Element(_), List::BLSG2Element(_)) => true,
                (List::Generic(t0), List::Generic(t1)) => t0
                    .map_or_else(|e0| e0, |s0| s0.first())
                    .type_eq(t1.map_or_else(|e1| e1, |s1| s1.first())),
                _ => false,
            },
            (Constant::Pair(a0, a1), Constant::Pair(b0, b1)) => a0.type_eq(b0) && a1.type_eq(b1),
            (Constant::PairData((d0, d1)), Constant::Pair(c0, c1))
            | (Constant::Pair(c0, c1), Constant::PairData((d0, d1))) => {
                c0.type_eq(&Constant::Data(d0)) && c1.type_eq(&Constant::Data(d1))
            }
            _ => false,
        }
    }
}

fn type_from_str<'a>(s: &str, arena: &'a Arena) -> Option<Constant<'a>> {
    let (main_ty, mut rest) = lex::word(s);

    let ret = match main_ty {
        "integer" => Constant::INTEGER_TYPE,
        "bytestring" => Constant::BYTES_TYPE,
        "string" => Constant::STRING_TYPE,
        "bool" => Constant::BOOLEAN_TYPE,
        "unit" => Constant::UNIT_TYPE,
        "data" => Constant::DATA_TYPE,
        "bls12_381_G1_element" => Constant::BLSG1_TYPE,
        "bls12_381_G2_element" => Constant::BLSG2_TYPE,
        "list" | "array" => {
            let (element_ty, new_rest) = lex::constant_type(rest)?;
            rest = new_rest;

            let list_ty = list_from_split(element_ty, "", arena).ok()?;
            if main_ty == "array" {
                Constant::Array(Array(list_ty))
            } else {
                Constant::List(list_ty)
            }
        }
        "pair" => {
            let (first_ty, new_rest) = lex::constant_type(rest)?;
            let (second_ty, new_rest) = lex::constant_type(new_rest)?;
            rest = new_rest;

            let first_const = type_from_str(first_ty, arena)?;
            let second_const = type_from_str(second_ty, arena)?;
            Constant::Pair(arena.alloc(first_const), arena.alloc(second_const))
        }

        _ => return None,
    };

    if !rest.is_empty() { None } else { Some(ret) }
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
        "bytestring" => {
            let hex = konst_word.strip_prefix("#").ok_or(ParseError::Bytestring)?;
            let value = const_hex::decode(hex).map_err(|_| ParseError::Bytestring)?;
            Constant::Bytes(arena.slice_fill(value))
        }
        "string" => {
            let (string, rest) = lex::string(konst).ok_or(ParseError::String)?;
            konst_rest = rest;
            let string = arena.string(&string);

            Constant::String(string)
        }

        "bool" => Constant::Boolean(match konst_word {
            "True" => true,
            "False" => false,
            _ => return Err(ParseError::Boolean),
        }),
        "unit" => {
            if konst_word == "()" {
                Constant::Unit
            } else {
                return Err(ParseError::Unit);
            }
        }
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
                Constant::List(list)
            } else {
                Constant::Array(Array(list))
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
    let mut words = items_str.split(',').map(str::trim);
    if items_str.trim().is_empty() {
        // If `""`, there will still ben an empty string in the iterator.
        words.next();
    }

    match ty_start {
        "integer" => words
            .map(integer)
            .collect::<Option<Vec<rug::Integer>>>()
            .ok_or(ParseError::Integer)
            .map(|ints| List::Integer(arena.integers(ints))),
        "data" => list_from_fn(items_str, data)
            .ok_or(ParseError::Data)
            .map(|data| List::Data(arena.datas(data))),
        "bls12_381_G1_element" => words
            .map(g1)
            .collect::<Option<Vec<g1::Projective>>>()
            .ok_or(ParseError::BLSG1Element)
            .map(|g1s| List::BLSG1Element(arena.slice_fill(g1s))),
        "bls12_381_G2_element" => words
            .map(g2)
            .collect::<Option<Vec<g2::Projective>>>()
            .ok_or(ParseError::BLSG2Element)
            .map(|g2s| List::BLSG2Element(arena.slice_fill(g2s))),
        "pair"
            if {
                let (first, rest) = lex::constant_type(ty_rest).ok_or(ParseError::Pair)?;
                let (second, rest) = lex::constant_type(rest).ok_or(ParseError::Pair)?;
                rest.is_empty() && first == "data" && second == "data"
            } =>
        {
            list_from_fn(items_str, |s| {
                let (pair_str, rest) = lex::group::<b'(', b')'>(s)?;
                let (key, r) = data(pair_str)?;
                let Some((value, "")) = data(r.strip_prefix(',')?.trim_start()) else {
                    return None;
                };
                Some(((key, value), rest))
            })
            .ok_or(ParseError::List)
            .map(|pairs| List::PairData(arena.pair_datas(pairs)))
        }

        "list" | "array" | "pair" | "bool" | "unit" | "string" | "bytestring" => {
            let list = list_from_fn(items_str, |s| from_split(ty, s, arena).ok())
                .ok_or(ParseError::List)?;
            if list.is_empty() {
                Ok(List::Generic(Err(arena.alloc(
                    type_from_str(ty, arena).ok_or(ParseError::UnknownType)?,
                ))))
            } else {
                Ok(List::Generic(Ok(Slice1::try_from_slice(
                    arena.slice_fill(list),
                )
                .expect("items is checked to be non-empty"))))
            }
        }
        _ => Err(ParseError::UnknownType),
    }
}

fn integer(s: &str) -> Option<rug::Integer> {
    rug::Integer::from_str_radix(s, 10).ok()
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
        rest = new_rest;
        items.push(item);
        if let Some(r) = rest.strip_prefix(',') {
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

// `TryFrom` implementations.

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

impl<'a> TryFrom<Constant<'a>> for Array<'a> {
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::Array(a) = value {
            Ok(a)
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<Constant<'a>> for &'a [rug::Integer] {
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::List(List::Integer(ints)) = value {
            Ok(ints)
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<Constant<'a>> for &'a [g1::Projective] {
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::List(List::BLSG1Element(projectives)) = value {
            Ok(projectives)
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<Constant<'a>> for &'a [g2::Projective] {
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::List(List::BLSG2Element(projectives)) = value {
            Ok(projectives)
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant<'_>> for Vec<Data> {
    type Error = ();

    fn try_from(value: Constant<'_>) -> Result<Self, Self::Error> {
        if let Constant::List(List::Data(datas)) = value {
            Ok(datas.to_vec())
        } else {
            Err(())
        }
    }
}

impl TryFrom<Constant<'_>> for Vec<(Data, Data)> {
    type Error = ();

    fn try_from(value: Constant<'_>) -> Result<Self, Self::Error> {
        if let Constant::List(List::PairData(pairs)) = value {
            Ok(pairs.to_vec())
        } else {
            Err(())
        }
    }
}

// TODO: Impl for Array.

impl<'a, A, B> TryFrom<Constant<'a>> for (A, B)
where
    A: TryFrom<Constant<'a>>,
    B: TryFrom<Constant<'a>>,
{
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::Pair(k, v) = value {
            let k = (*k).try_into().map_err(|_| ())?;
            let v = (*v).try_into().map_err(|_| ())?;
            Ok((k, v))
        } else if let Constant::PairData((a, b)) = value {
            let k = A::try_from(Constant::Data(a)).map_err(|_| ())?;
            let v = B::try_from(Constant::Data(b)).map_err(|_| ())?;
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

impl<'a> TryFrom<Constant<'a>> for &'a g1::Projective {
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::BLSG1Element(p) = value {
            Ok(p)
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

impl<'a> TryFrom<Constant<'a>> for &'a g2::Projective {
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::BLSG2Element(p) = value {
            Ok(p)
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

impl<'a> TryFrom<Constant<'a>> for &'a bwst::miller_loop::Result {
    type Error = ();

    fn try_from(value: Constant<'a>) -> Result<Self, Self::Error> {
        if let Constant::MillerLoopResult(r) = value {
            Ok(r)
        } else {
            Err(())
        }
    }
}

// `Into` implementations.

impl<'a> Into<Constant<'a>> for &'a rug::Integer {
    fn into(self) -> Constant<'a> {
        Constant::Integer(self)
    }
}

impl<'a> Into<Constant<'a>> for &'a [u8] {
    fn into(self) -> Constant<'a> {
        Constant::Bytes(self)
    }
}

impl<'a> Into<Constant<'a>> for &'a str {
    fn into(self) -> Constant<'a> {
        Constant::String(self)
    }
}

impl Into<Constant<'_>> for () {
    fn into(self) -> Constant<'static> {
        Constant::Unit
    }
}

impl Into<Constant<'_>> for bool {
    fn into(self) -> Constant<'static> {
        Constant::Boolean(self)
    }
}

impl<'a> Into<Constant<'a>> for List<'a> {
    fn into(self) -> Constant<'a> {
        Constant::List(self)
    }
}

impl<'a> Into<Constant<'a>> for Array<'a> {
    fn into(self) -> Constant<'a> {
        Constant::Array(self)
    }
}

impl<'a> Into<Constant<'a>> for &'a Data {
    fn into(self) -> Constant<'a> {
        Constant::Data(self)
    }
}

impl<'a> Into<Constant<'a>> for &'a [Data] {
    fn into(self) -> Constant<'a> {
        Constant::List(List::Data(self))
    }
}

impl<'a> Into<Constant<'a>> for &'a [(Data, Data)] {
    fn into(self) -> Constant<'a> {
        Constant::List(List::PairData(self))
    }
}

impl<'a> Into<Constant<'a>> for (&'a Constant<'a>, &'a Constant<'a>) {
    fn into(self) -> Constant<'a> {
        Constant::Pair(self.0, self.1)
    }
}

// `Output` trait for owned types.

// We should have something for all `T` where `&'a T: Into<Constant<'a>>` and `T: Copy`, but this
// is a conflicting impl for now.

impl<'a> Output<'a> for rug::Integer {
    fn into(value: Self, arena: &'a self::Arena) -> Option<crate::machine::Value<'a>> {
        Some(crate::machine::Value::Constant(Constant::Integer(
            arena.integer(value),
        )))
    }
}

impl<'a> Output<'a> for Vec<u8> {
    fn into(value: Self, arena: &'a self::Arena) -> Option<crate::machine::Value<'a>> {
        Some(crate::machine::Value::Constant(Constant::Bytes(
            arena.slice_fill(value.into_iter()),
        )))
    }
}

impl<'a> Output<'a> for String {
    fn into(value: Self, arena: &'a self::Arena) -> Option<crate::machine::Value<'a>> {
        Some(crate::machine::Value::Constant(Constant::String(
            arena.string(&value),
        )))
    }
}

impl<'a> Output<'a> for Data {
    fn into(value: Self, arena: &'a self::Arena) -> Option<crate::machine::Value<'a>> {
        Some(crate::machine::Value::Constant(Constant::Data(
            arena.data(value),
        )))
    }
}

impl<'a, A, B> Output<'a> for (A, B)
where
    A: Into<Constant<'a>>,
    B: Into<Constant<'a>>,
{
    fn into(value: Self, arena: &'a self::Arena) -> Option<crate::machine::Value<'a>> {
        Some(crate::machine::Value::Constant(Constant::Pair(
            arena.alloc(value.0.into()),
            arena.alloc(value.1.into()),
        )))
    }
}

impl<'a> Output<'a> for (rug::Integer, &'a [Data]) {
    fn into(value: Self, arena: &'a self::Arena) -> Option<crate::machine::Value<'a>> {
        Some(crate::machine::Value::Constant(Constant::Pair(
            arena.alloc(Constant::Integer(arena.integer(value.0))),
            arena.alloc(Constant::List(List::Data(value.1))),
        )))
    }
}

impl<'a> Output<'a> for g1::Projective {
    fn into(value: Self, arena: &'a self::Arena) -> Option<crate::machine::Value<'a>> {
        Some(crate::machine::Value::Constant(Constant::BLSG1Element(
            arena.alloc(value),
        )))
    }
}

impl<'a> Output<'a> for g2::Projective {
    fn into(value: Self, arena: &'a self::Arena) -> Option<crate::machine::Value<'a>> {
        Some(crate::machine::Value::Constant(Constant::BLSG2Element(
            arena.alloc(value),
        )))
    }
}

impl<'a> Output<'a> for bwst::miller_loop::Result {
    fn into(value: Self, arena: &'a self::Arena) -> Option<crate::machine::Value<'a>> {
        Some(crate::machine::Value::Constant(Constant::MillerLoopResult(
            arena.alloc(value),
        )))
    }
}
