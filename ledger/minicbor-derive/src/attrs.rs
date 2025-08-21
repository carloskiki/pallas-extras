//! Attribute handling.

pub mod codec;
pub mod encoding;
pub mod idx;
pub mod typeparam;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::iter;

use syn::spanned::Spanned;
use syn::{LitInt, LitStr};

pub use codec::CustomCodec;
pub use encoding::Encoding;
pub use idx::Idx;
pub use typeparam::TypeParams;

/// Recognised attributes.
#[derive(Debug, Clone)]
pub struct Attributes {
    level: Level,
    attrs: HashMap<Kind, Value>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    Borrow,
    Codec,
    Encoding,
    Index,
    IndexOnly,
    Transparent,
    TypeParam,
    Nil,
    IsNil,
    HasNil,
    ContextBound,
    CborLen,
    Tag,
    Skip,
    Flat,
    Default,
    Era,
}

#[derive(Debug, Clone)]
enum Value {
    Borrow(BTreeSet<syn::Lifetime>, proc_macro2::Span),
    Codec(CustomCodec, proc_macro2::Span),
    Encoding(Encoding, proc_macro2::Span),
    Index(Idx, proc_macro2::Span),
    IndexOnly(proc_macro2::Span),
    Transparent(proc_macro2::Span),
    TypeParam(TypeParams, proc_macro2::Span),
    Nil(syn::ExprPath, proc_macro2::Span),
    IsNil(syn::ExprPath, proc_macro2::Span),
    HasNil(proc_macro2::Span),
    ContextBound(HashSet<syn::TraitBound>, proc_macro2::Span),
    CborLen(syn::ExprPath, proc_macro2::Span),
    Tag(u64, proc_macro2::Span),
    Skip(proc_macro2::Span),
    Flat(proc_macro2::Span),
    Default(proc_macro2::Span),
    Era(syn::Ident, proc_macro2::Span)
}

#[derive(Debug, Copy, Clone)]
pub enum Level {
    Enum,
    Struct,
    Variant,
    Field,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Enum => f.write_str("enum"),
            Level::Struct => f.write_str("struct"),
            Level::Variant => f.write_str("variant"),
            Level::Field => f.write_str("field"),
        }
    }
}

impl Attributes {
    fn new(l: Level) -> Self {
        Self {
            level: l,
            attrs: HashMap::new(),
        }
    }

    pub fn try_from_iter<'a, I>(l: Level, attrs: I) -> syn::Result<Self>
    where
        I: IntoIterator<Item = &'a syn::Attribute>,
    {
        let mut this = Self::new(l);
        for m in attrs.into_iter().map(|a| Self::try_from(l, a)) {
            let m = m?;
            for (k, v) in m.attrs.into_iter() {
                this.try_insert(k, v)?;
            }
        }
        if let Some(Value::IsNil(_, s)) = this.get(Kind::IsNil) {
            return Err(syn::Error::new(*s, "`is_nil` requires `encode_with`"));
        }
        if let Some(Value::Nil(_, s)) = this.get(Kind::Nil) {
            return Err(syn::Error::new(*s, "`nil` requires `decode_with`"));
        }
        if let Some(Value::HasNil(s)) = this.get(Kind::HasNil) {
            return Err(syn::Error::new(*s, "`has_nil` requires `with`"));
        }
        if let Some(Value::Tag(_, s)) = this.get(Kind::Tag) {
            if this.contains_key(Kind::IndexOnly) {
                return Err(syn::Error::new(
                    *s,
                    "`tag` and `index_only` are mutually exclusive",
                ));
            }
            if this.contains_key(Kind::Transparent) {
                return Err(syn::Error::new(
                    *s,
                    "`tag` and `transparent` are mutually exclusive",
                ));
            }
        }
        if let Some(Value::Skip(s)) = this.get(Kind::Skip) {
            if this.attrs.len() > 1 {
                return Err(syn::Error::new(
                    *s,
                    "`skip` does not allow other attributes",
                ));
            }
        }
        if let Some(Value::Flat(_)) = this.get(Kind::Flat) {
            if let Some(Value::Encoding(Encoding::Map, s)) = this.get(Kind::Encoding) {
                return Err(syn::Error::new(
                    *s,
                    "flat enum does not support map encoding",
                ));
            }
        }
        Ok(this)
    }

    fn try_from(l: Level, a: &syn::Attribute) -> syn::Result<Self> {
        let mut attrs = Self::new(l);

        // #[n(...)]
        if a.path().is_ident("n") {
            let idx = parse_i64_arg(a).map(Idx::N)?;
            attrs.try_insert(Kind::Index, Value::Index(idx, a.path().span()))?;
            return Ok(attrs);
        }

        // #[b(...)]
        if a.path().is_ident("b") {
            let idx = parse_i64_arg(a).map(Idx::B)?;
            attrs.try_insert(Kind::Index, Value::Index(idx, a.path().span()))?;
            return Ok(attrs);
        }

        // #[cbor(...)]
        if !a.path().is_ident("cbor") {
            return Ok(Self::new(l));
        }

        a.parse_nested_meta(|meta| {
            if meta.path.is_ident("index_only") {
                attrs.try_insert(Kind::IndexOnly, Value::IndexOnly(meta.path.span()))?
            } else if meta.path.is_ident("transparent") {
                attrs.try_insert(Kind::Transparent, Value::Transparent(meta.path.span()))?
            } else if meta.path.is_ident("map") {
                attrs.try_insert(
                    Kind::Encoding,
                    Value::Encoding(Encoding::Map, meta.path.span()),
                )?
            } else if meta.path.is_ident("array") {
                attrs.try_insert(
                    Kind::Encoding,
                    Value::Encoding(Encoding::Array, meta.path.span()),
                )?
            } else if meta.path.is_ident("has_nil") {
                attrs.try_insert(Kind::HasNil, Value::HasNil(meta.path.span()))?
            } else if meta.path.is_ident("encode_with") {
                let s: LitStr = meta.value()?.parse()?;
                let c = CustomCodec::Encode(codec::Encode {
                    encode: s.parse()?,
                    is_nil: None,
                });
                attrs.try_insert(Kind::Codec, Value::Codec(c, meta.path.span()))?
            } else if meta.path.is_ident("is_nil") {
                let s: LitStr = meta.value()?.parse()?;
                attrs.try_insert(Kind::IsNil, Value::IsNil(s.parse()?, meta.path.span()))?
            } else if meta.path.is_ident("decode_with") {
                let s: LitStr = meta.value()?.parse()?;
                let c = CustomCodec::Decode(codec::Decode {
                    decode: s.parse()?,
                    nil: None,
                });
                attrs.try_insert(Kind::Codec, Value::Codec(c, meta.path.span()))?
            } else if meta.path.is_ident("nil") {
                let s: LitStr = meta.value()?.parse()?;
                attrs.try_insert(Kind::Nil, Value::Nil(s.parse()?, meta.path.span()))?
            } else if meta.path.is_ident("with") {
                let s: LitStr = meta.value()?.parse()?;
                let c = CustomCodec::Module(s.parse()?, false);
                attrs.try_insert(Kind::Codec, Value::Codec(c, meta.path.span()))?
            } else if meta.path.is_ident("borrow") {
                let mut l = BTreeSet::new();
                if meta.input.peek(syn::Token!(=)) {
                    let s: LitStr = meta.value()?.parse()?;
                    for b in s.value().split('+').filter(|b| !b.is_empty()) {
                        l.insert(syn::parse_str::<syn::Lifetime>(b.trim())?);
                    }
                }
                attrs.try_insert(Kind::Borrow, Value::Borrow(l, meta.path.span()))?
            } else if meta.path.is_ident("encode_bound") {
                let s: LitStr = meta.value()?.parse()?;
                let t: syn::TypeParam = s.parse()?;
                let b = TypeParams::Encode(iter::once((t.ident.clone(), t)).collect());
                attrs.try_insert(Kind::TypeParam, Value::TypeParam(b, meta.path.span()))?
            } else if meta.path.is_ident("decode_bound") {
                let s: LitStr = meta.value()?.parse()?;
                let t: syn::TypeParam = s.parse()?;
                let b = TypeParams::Decode(iter::once((t.ident.clone(), t)).collect());
                attrs.try_insert(Kind::TypeParam, Value::TypeParam(b, meta.path.span()))?
            } else if meta.path.is_ident("cbor_len_bound") {
                let s: LitStr = meta.value()?.parse()?;
                let t: syn::TypeParam = s.parse()?;
                let b = TypeParams::Length(iter::once((t.ident.clone(), t)).collect());
                attrs.try_insert(Kind::TypeParam, Value::TypeParam(b, meta.path.span()))?
            } else if meta.path.is_ident("bound") {
                let s: LitStr = meta.value()?.parse()?;
                let t: syn::TypeParam = s.parse()?;
                let m = iter::once((t.ident.clone(), t)).collect::<HashMap<_, _>>();
                let b = TypeParams::All {
                    encode: m.clone(),
                    length: m.clone(),
                    decode: m,
                };
                attrs.try_insert(Kind::TypeParam, Value::TypeParam(b, meta.path.span()))?
            } else if meta.path.is_ident("context_bound") {
                let s: LitStr = meta.value()?.parse()?;
                let mut h = HashSet::new();
                for b in s.value().split('+').filter(|b| !b.is_empty()) {
                    h.insert(syn::parse_str::<syn::TraitBound>(b.trim())?);
                }
                attrs.try_insert(Kind::ContextBound, Value::ContextBound(h, meta.path.span()))?
            } else if meta.path.is_ident("cbor_len") {
                let s: LitStr = meta.value()?.parse()?;
                attrs.try_insert(Kind::CborLen, Value::CborLen(s.parse()?, meta.path.span()))?
            } else if meta.path.is_ident("n") {
                let content;
                syn::parenthesized!(content in meta.input);
                let n: LitInt = content.parse()?;
                let i = parse_int(&n).map(Idx::N)?;
                attrs.try_insert(Kind::Index, Value::Index(i, meta.path.span()))?
            } else if meta.path.is_ident("b") {
                let content;
                syn::parenthesized!(content in meta.input);
                let n: LitInt = content.parse()?;
                let i = parse_int(&n).map(Idx::B)?;
                attrs.try_insert(Kind::Index, Value::Index(i, meta.path.span()))?
            } else if meta.path.is_ident("tag") {
                let content;
                syn::parenthesized!(content in meta.input);
                let n: LitInt = content.parse()?;
                let i = n.base10_parse()?;
                attrs.try_insert(Kind::Tag, Value::Tag(i, meta.path.span()))?
            } else if meta.path.is_ident("skip") {
                attrs.try_insert(Kind::Skip, Value::Skip(meta.path.span()))?
            } else if meta.path.is_ident("flat") {
                attrs.try_insert(Kind::Flat, Value::Flat(meta.path.span()))?
            } else if meta.path.is_ident("default") {
                attrs.try_insert(Kind::Default, Value::Default(meta.path.span()))?
            } else if meta.path.is_ident("era") {
                let s: LitStr = meta.value()?.parse()?;
                let ident = syn::parse_str::<syn::Ident>(&s.value())?;
                attrs.try_insert(
                    Kind::Era,
                    Value::Era(ident, meta.path.span()),
                )?
            } else {
                return Err(meta.error("unsupported attribute"));
            }
            Ok(())
        })?;

        Ok(attrs)
    }

    pub fn span(&self, k: Kind) -> Option<proc_macro2::Span> {
        self.get(k).map(|v| v.span())
    }

    pub fn borrow(&self) -> Option<&BTreeSet<syn::Lifetime>> {
        self.get(Kind::Borrow).and_then(|v| v.borrow())
    }

    pub fn encoding(&self) -> Option<Encoding> {
        self.get(Kind::Encoding).and_then(|v| v.encoding())
    }

    pub fn index(&self) -> Option<Idx> {
        self.get(Kind::Index).and_then(|v| v.index())
    }

    pub fn codec(&self) -> Option<&CustomCodec> {
        self.get(Kind::Codec).and_then(|v| v.codec())
    }

    pub fn type_params(&self) -> Option<&TypeParams> {
        self.get(Kind::TypeParam).and_then(|v| v.type_params())
    }

    pub fn context_bound(&self) -> Option<impl Iterator<Item = &syn::TraitBound>> {
        self.get(Kind::ContextBound).and_then(|v| v.context_bound())
    }

    pub fn transparent(&self) -> bool {
        self.contains_key(Kind::Transparent)
    }

    pub fn index_only(&self) -> bool {
        self.contains_key(Kind::IndexOnly)
    }

    pub fn cbor_len(&self) -> Option<&syn::ExprPath> {
        self.get(Kind::CborLen).and_then(|v| v.cbor_len())
    }

    pub fn tag(&self) -> Option<u64> {
        self.get(Kind::Tag).and_then(|v| v.tag())
    }

    pub fn skip(&self) -> bool {
        self.contains_key(Kind::Skip)
    }

    pub fn flat(&self) -> bool {
        self.contains_key(Kind::Flat)
    }

    pub fn default(&self) -> bool {
        self.contains_key(Kind::Default)
    }
    
    pub fn era(&self) -> Option<&syn::Ident> {
        self.get(Kind::Era).and_then(|v| v.era())
    }

    fn contains_key(&self, k: Kind) -> bool {
        self.attrs.contains_key(&k)
    }

    fn get(&self, k: Kind) -> Option<&Value> {
        self.attrs.get(&k)
    }

    fn get_mut(&mut self, k: Kind) -> Option<&mut Value> {
        self.attrs.get_mut(&k)
    }

    fn remove(&mut self, k: Kind) -> Option<Value> {
        self.attrs.remove(&k)
    }

    fn try_insert(&mut self, key: Kind, mut val: Value) -> syn::Result<()> {
        match self.level {
            Level::Struct => match key {
                Kind::Encoding | Kind::Transparent | Kind::ContextBound | Kind::Tag | Kind::Era => {}
                Kind::Borrow
                | Kind::TypeParam
                | Kind::Codec
                | Kind::Index
                | Kind::IndexOnly
                | Kind::Nil
                | Kind::IsNil
                | Kind::HasNil
                | Kind::CborLen
                | Kind::Skip
                | Kind::Flat
                | Kind::Default => {
                    let msg = format!("attribute is not supported on {}-level", self.level);
                    return Err(syn::Error::new(val.span(), msg));
                }
            },
            Level::Field => match key {
                Kind::TypeParam
                | Kind::Borrow
                | Kind::Codec
                | Kind::Index
                | Kind::Nil
                | Kind::IsNil
                | Kind::HasNil
                | Kind::CborLen
                | Kind::Tag
                | Kind::Skip
                | Kind::Default => {}
                Kind::Encoding
                | Kind::IndexOnly
                | Kind::Transparent
                | Kind::ContextBound
                | Kind::Flat
                | Kind::Era => {
                    let msg = format!("attribute is not supported on {}-level", self.level);
                    return Err(syn::Error::new(val.span(), msg));
                }
            },
            Level::Enum => match key {
                Kind::Encoding | Kind::IndexOnly | Kind::ContextBound | Kind::Tag | Kind::Flat | Kind::Era => {}
                Kind::Borrow
                | Kind::TypeParam
                | Kind::Codec
                | Kind::Index
                | Kind::Transparent
                | Kind::Nil
                | Kind::IsNil
                | Kind::HasNil
                | Kind::CborLen
                | Kind::Skip
                | Kind::Default => {
                    let msg = format!("attribute is not supported on {}-level", self.level);
                    return Err(syn::Error::new(val.span(), msg));
                }
            },
            Level::Variant => match key {
                Kind::Encoding | Kind::Index | Kind::Tag => {}
                Kind::Borrow
                | Kind::TypeParam
                | Kind::Codec
                | Kind::IndexOnly
                | Kind::Transparent
                | Kind::Nil
                | Kind::IsNil
                | Kind::HasNil
                | Kind::ContextBound
                | Kind::CborLen
                | Kind::Skip
                | Kind::Flat
                | Kind::Default
                | Kind::Era => {
                    let msg = format!("attribute is not supported on {}-level", self.level);
                    return Err(syn::Error::new(val.span(), msg));
                }
            },
        }
        if self.contains_key(key) {
            if let Some(Value::Codec(cc, _)) = self.get_mut(key) {
                let s = val.span();
                match (val, &cc) {
                    (Value::Codec(CustomCodec::Encode(e), _), CustomCodec::Decode(d)) => {
                        let d = codec::Decode {
                            decode: d.decode.clone(),
                            nil: d.nil.clone(),
                        };
                        *cc = CustomCodec::Both(Box::new(e), Box::new(d));
                        return Ok(());
                    }
                    (Value::Codec(CustomCodec::Decode(d), _), CustomCodec::Encode(e)) => {
                        let e = codec::Encode {
                            encode: e.encode.clone(),
                            is_nil: e.is_nil.clone(),
                        };
                        *cc = CustomCodec::Both(Box::new(e), Box::new(d));
                        return Ok(());
                    }
                    _ => return Err(syn::Error::new(s, "duplicate attribute")),
                }
            } else if let Some(Value::TypeParam(cb, _)) = self.get_mut(key) {
                let s = val.span();
                if let Value::TypeParam(p, _) = val {
                    cb.try_merge(s, p)?;
                    return Ok(());
                }
                return Err(syn::Error::new(s, "duplicate attribute"));
            } else if let Some(Value::ContextBound(cb, _)) = self.get_mut(key) {
                let s = val.span();
                if let Value::ContextBound(x, _) = val {
                    cb.extend(x);
                    return Ok(());
                }
                return Err(syn::Error::new(s, "duplicate attribute"));
            } else {
                return Err(syn::Error::new(val.span(), "duplicate attribute"));
            }
        }
        match &mut val {
            Value::IsNil(is_nil, s) => match self.get_mut(Kind::Codec) {
                Some(Value::Codec(CustomCodec::Encode(e), _)) => {
                    if e.is_nil.is_some() {
                        return Err(syn::Error::new(*s, "duplicate attribute"));
                    }
                    e.is_nil = Some(is_nil.clone());
                    return Ok(());
                }
                Some(Value::Codec(CustomCodec::Both(e, _), _)) => {
                    if e.is_nil.is_some() {
                        return Err(syn::Error::new(*s, "duplicate attribute"));
                    }
                    e.is_nil = Some(is_nil.clone());
                    return Ok(());
                }
                _ => {}
            },
            Value::Nil(nil, s) => match self.get_mut(Kind::Codec) {
                Some(Value::Codec(CustomCodec::Decode(d), _)) => {
                    if d.nil.is_some() {
                        return Err(syn::Error::new(*s, "duplicate attribute"));
                    }
                    d.nil = Some(nil.clone());
                    return Ok(());
                }
                Some(Value::Codec(CustomCodec::Both(_, d), _)) => {
                    if d.nil.is_some() {
                        return Err(syn::Error::new(*s, "duplicate attribute"));
                    }
                    d.nil = Some(nil.clone());
                    return Ok(());
                }
                _ => {}
            },
            Value::HasNil(s) => {
                if let Some(Value::Codec(CustomCodec::Module(_, b), _)) = self.get_mut(Kind::Codec)
                {
                    if *b {
                        return Err(syn::Error::new(*s, "duplicate attribute"));
                    }
                    *b = true;
                    return Ok(());
                }
            }
            Value::Codec(CustomCodec::Encode(e), s) => {
                if let Some(Value::IsNil(is_nil, _)) = self.remove(Kind::IsNil) {
                    if e.is_nil.is_some() {
                        return Err(syn::Error::new(*s, "duplicate attribute"));
                    }
                    e.is_nil = Some(is_nil)
                }
            }
            Value::Codec(CustomCodec::Decode(d), s) => {
                if let Some(Value::Nil(nil, _)) = self.remove(Kind::Nil) {
                    if d.nil.is_some() {
                        return Err(syn::Error::new(*s, "duplicate attribute"));
                    }
                    d.nil = Some(nil)
                }
            }
            Value::Codec(CustomCodec::Both(e, d), s) => {
                if let Some(Value::IsNil(is_nil, _)) = self.remove(Kind::IsNil) {
                    if e.is_nil.is_some() {
                        return Err(syn::Error::new(*s, "duplicate attribute"));
                    }
                    e.is_nil = Some(is_nil)
                }
                if let Some(Value::Nil(nil, _)) = self.remove(Kind::Nil) {
                    if d.nil.is_some() {
                        return Err(syn::Error::new(*s, "duplicate attribute"));
                    }
                    d.nil = Some(nil)
                }
            }
            Value::Codec(CustomCodec::Module(_, b), s) => {
                if let Some(Value::HasNil(_)) = self.remove(Kind::HasNil) {
                    if *b {
                        return Err(syn::Error::new(*s, "duplicate attribute"));
                    }
                    *b = true
                }
                if self.contains_key(Kind::CborLen) {
                    return Err(syn::Error::new(
                        *s,
                        "`with` and `cbor_len` are mutually exclusive",
                    ));
                }
            }
            Value::CborLen(_, s) => {
                if let Some(Value::Codec(c, _)) = self.get(Kind::Codec) {
                    if c.is_module() {
                        return Err(syn::Error::new(
                            *s,
                            "`cbor_len` and `with` are mutually exclusive",
                        ));
                    }
                }
            }
            Value::Borrow(_, s) => {
                if let Some(idx) = self.index() {
                    if idx.is_b() {
                        return Err(syn::Error::new(
                            *s,
                            "`borrow` and `b` are mutually exclusive",
                        ));
                    }
                }
            }
            Value::Index(idx, s) if idx.is_b() => {
                if self.contains_key(Kind::Borrow) {
                    return Err(syn::Error::new(
                        *s,
                        "`b` and `borrow` are mutually exclusive",
                    ));
                }
            }
            _ => {}
        }
        self.attrs.insert(key, val);
        Ok(())
    }
}

impl Value {
    fn span(&self) -> proc_macro2::Span {
        match self {
            Value::Borrow(_, s) => *s,
            Value::TypeParam(_, s) => *s,
            Value::Codec(_, s) => *s,
            Value::Encoding(_, s) => *s,
            Value::Index(_, s) => *s,
            Value::IndexOnly(s) => *s,
            Value::Transparent(s) => *s,
            Value::Nil(_, s) => *s,
            Value::IsNil(_, s) => *s,
            Value::HasNil(s) => *s,
            Value::ContextBound(_, s) => *s,
            Value::CborLen(_, s) => *s,
            Value::Tag(_, s) => *s,
            Value::Skip(s) => *s,
            Value::Flat(s) => *s,
            Value::Default(s) => *s,
            Value::Era(_, s) => *s,
        }
    }

    fn borrow(&self) -> Option<&BTreeSet<syn::Lifetime>> {
        if let Value::Borrow(l, _) = self {
            Some(l)
        } else {
            None
        }
    }

    fn index(&self) -> Option<Idx> {
        if let Value::Index(i, _) = self {
            Some(*i)
        } else {
            None
        }
    }

    fn codec(&self) -> Option<&CustomCodec> {
        if let Value::Codec(c, _) = self {
            Some(c)
        } else {
            None
        }
    }

    fn encoding(&self) -> Option<Encoding> {
        if let Value::Encoding(e, _) = self {
            Some(*e)
        } else {
            None
        }
    }

    fn type_params(&self) -> Option<&TypeParams> {
        if let Value::TypeParam(t, _) = self {
            Some(t)
        } else {
            None
        }
    }

    fn context_bound(&self) -> Option<impl Iterator<Item = &syn::TraitBound>> {
        if let Value::ContextBound(x, _) = self {
            Some(x.iter())
        } else {
            None
        }
    }

    fn cbor_len(&self) -> Option<&syn::ExprPath> {
        if let Value::CborLen(x, _) = self {
            Some(x)
        } else {
            None
        }
    }

    fn tag(&self) -> Option<u64> {
        if let Value::Tag(x, _) = self {
            Some(*x)
        } else {
            None
        }
    }

    fn era(&self) -> Option<&syn::Ident> {
        if let Value::Era(ident, _) = self {
            Some(ident)
        } else {
            None
        }
    }
}

fn parse_i64_arg(a: &syn::Attribute) -> syn::Result<i64> {
    parse_int(&a.parse_args()?)
}

fn parse_int(n: &syn::LitInt) -> syn::Result<i64> {
    n.base10_parse()
        .map_err(|_| syn::Error::new(n.span(), "expected `i64` value"))
}
