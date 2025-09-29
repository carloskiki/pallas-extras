use crate::ast::{struct_pattern, structure::Struct};

use super::{Encoding, Field, attr_index, encoding_update, meta_index, parse_fields};
use quote::quote;

#[derive(Debug, Clone)]
pub enum Enum {
    Normal {
        encoding: Encoding,
        variants: Vec<Variant>,
    },
    Index {
        variants: Vec<IndexVariant>,
    },
    Flat {
        variants: Vec<FlatVariant>,
    },
}

impl Enum {
    pub fn normal(input: &syn::DataEnum, encoding: Option<Encoding>) -> syn::Result<Self> {
        Ok(Enum::Normal {
            encoding: encoding.unwrap_or_default(),
            variants: input
                .variants
                .iter()
                .map(Variant::try_from)
                .collect::<Result<_, _>>()?,
        })
    }

    pub fn index(input: &syn::DataEnum) -> syn::Result<Self> {
        Ok(Enum::Index {
            variants: input
                .variants
                .iter()
                .map(IndexVariant::try_from)
                .collect::<Result<_, _>>()?,
        })
    }

    pub fn flat(input: &syn::DataEnum) -> syn::Result<Self> {
        Ok(Enum::Flat {
            variants: input
                .variants
                .iter()
                .map(FlatVariant::try_from)
                .collect::<Result<_, _>>()?,
        })
    }

    pub fn decode(self) -> proc_macro2::TokenStream {
        match self {
            Enum::Normal { encoding, variants } => {
                let match_arms = variants.into_iter().map(|v| {
                    let index = v.index as i64;
                    let encoding = v.encoding.unwrap_or(encoding);
                    let _struct = Struct {
                        encoding,
                        fields: v.fields,
                    };
                    
                    let struct_proc = _struct.decode(Some(v.ident));
                    
                    quote! {
                        #index => {
                            #struct_proc
                        }
                    }
                });
                quote! {
                    {
                        let array_len = d.array()?;
                        if array_len != 2 {
                            return Err(::minicbor::decode::Error::message(format!("expected array of length 2, found {}", array_len)));
                        }
                        let index = d.i64()?;
                        match index {
                            #(#match_arms)*
                            _ => return Err(::minicbor::decode::Error::message(format!("unknown enum index {}", index))),
                        }
                    }
                }
            },
            Enum::Index { variants } => {
                let match_arms = variants.iter().map(|v| {
                    let name = &v.ident;
                    let index = v.index;
                    quote! {
                        #index => Self::#name {},
                    }
                });
                quote! {
                    {
                        let index = d.i64()?;
                        match index {
                            #(#match_arms)*
                            _ => return Err(::minicbor::decode::Error::message(format!("unknown enum index {}", index))),
                        }
                    }
                }
            },
            Enum::Flat { variants } => {
                let match_arms = variants.into_iter().map(|v| {
                    let index = v.index as i64;
                    let array_len = v.fields.last().map_or(0, |f| f.index + 1) as u64 + 1;
                    let match_pattern = struct_pattern(v.ident, &v.fields);
                    let mut fields = v.fields.into_iter().peekable();
                    let field_procedures = (0..(array_len - 1)).map(|i| {
                        let field = fields.peek().expect("the last index matches the last field");
                        if field.index as u64 == i {
                            let field = fields.next().expect("peeked");
                            field.decode()
                        } else {
                            quote ! {
                                d.skip()?;
                            }
                        }
                    });
                    quote! {
                        #index => {
                            if array_len != #array_len {
                                return Err(::minicbor::decode::Error::message(format!("expected array of length {}, found {}", #array_len, array_len)));
                            }
                            #(#field_procedures)*
                            Self::#match_pattern
                        }
                    }
                });
                quote! {
                    {
                        
                        let array_len = d.array()?;
                        let index = d.i64()?;
                        match index {
                            #(#match_arms)*
                            _ => return Err(cbor_event::Error::CustomError(format!("unknown enum index {}", index))),
                        }
                    }
                }
            },
        }
    }

    pub fn encode(self) -> proc_macro2::TokenStream {
        match self {
            Enum::Normal { encoding, variants } => {
                let match_arms = variants.into_iter().map(|v| {
                    let index = v.index as i64;
                    let encoding = v.encoding.unwrap_or(encoding);
                    let match_pattern = struct_pattern(v.ident, &v.fields);
                    let _struct = Struct {
                        encoding,
                        fields: v.fields,
                    };
                    
                    let struct_proc = _struct.encode(false);
                    
                    quote! {
                        #match_pattern => {
                            e.i64(#index)?;
                            #struct_proc
                        }
                    }
                });
                quote! {
                    e.array(2)?;
                    match self {
                        #(#match_arms)*
                    }
                }
            },
            Enum::Index { variants } => {
                let match_arms = variants.iter().map(|v| {
                    let name = &v.ident;
                    let index = v.index;
                    quote! {
                        Self::#name { .. } => #index,
                    }
                });
                quote! {
                    e.i64(match self {
                        #(#match_arms)*
                    })?;
                }
            },
            Enum::Flat { variants } => {
                let match_arms = variants.into_iter().map(|v| {
                    let index = v.index as i64;
                    let array_len = v.fields.last().map_or(0, |f| f.index + 1) as u64 + 1;
                    let match_pattern = struct_pattern(v.ident, &v.fields);
                    let mut fields = v.fields.into_iter().peekable();
                    let field_procedures = (0..array_len).map(|i| {
                        let field = fields.peek().expect("the last index matches the last field");
                        if field.index as u64 == i {
                            let field = fields.next().expect("peeked");
                            field.encode(false)
                        } else {
                            quote! {
                                e.null()?;
                            }
                        }
                    });
                    quote! {
                        #match_pattern => {
                            e.array(#array_len)?;
                            e.i64(#index)?;
                            #(#field_procedures)*
                        }
                    }
                });
                quote! {
                    match self {
                        #(#match_arms)*
                    }
                }
            },
        }
    }

    pub fn len(self) -> proc_macro2::TokenStream {
        match self {
            Enum::Normal { encoding, variants } => {
                let match_arms = variants.into_iter().map(|v| {
                    let index = v.index as i64;
                    let encoding = v.encoding.unwrap_or(encoding);
                    let match_pattern = struct_pattern(v.ident, &v.fields);
                    let _struct = Struct {
                        encoding,
                        fields: v.fields,
                    };
                    
                    let struct_proc = _struct.len(false);
                    
                    quote! {
                        #match_pattern => {
                            #index.cbor_len(ctx) + #struct_proc
                        }
                    }
                });
                quote! {
                    2.cbor_len(ctx) + match self {
                        #(#match_arms)*
                    }
                }
            },
            Enum::Index { variants } => {
                let match_arms = variants.iter().map(|v| {
                    let name = &v.ident;
                    let index = v.index;
                    quote! {
                        Self::#name { .. } => #index.cbor_len(ctx),
                    }
                });
                quote! {
                    match self {
                        #(#match_arms)*
                    }
                }
            },
            Enum::Flat { variants } => {
                let match_arms = variants.into_iter().map(|v| {
                    let index = v.index as i64;
                    let array_len = v.fields.last().map_or(0, |f| f.index + 1) as u64 + 1;
                    let match_pattern = struct_pattern(v.ident, &v.fields);
                    let mut fields = v.fields.into_iter().peekable();
                    let field_lens = (0..array_len).map(|i| {
                        let field = fields.peek().expect("the last index matches the last field");
                        if field.index as u64 == i {
                            let field = fields.next().expect("peeked");
                            field.len(false)
                        } else {
                            quote! { 1 } // null
                        }
                    });
                    quote! {
                        #match_pattern => {
                            #array_len.cbor_len(ctx) +
                            #index.cbor_len(ctx) +
                            #( #field_lens )+*
                        }
                    }
                });
                quote! {
                    match self {
                        #(#match_arms)*
                    }
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub index: isize,
    pub encoding: Option<Encoding>,
    pub fields: Vec<Field>,
    pub ident: syn::Ident,
}

impl TryFrom<&syn::Variant> for Variant {
    type Error = syn::Error;

    fn try_from(input: &syn::Variant) -> Result<Self, Self::Error> {
        let mut index = None;
        let mut encoding = None;

        for attr in &input.attrs {
            if attr_index(&mut index, attr, false)? {
                continue;
            }
            if !attr.path().is_ident("cbor") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta_index(&mut index, &meta, false)? || encoding_update(&mut encoding, &meta)? {
                    return Ok(());
                }
                Err(meta.error("unknown attribute"))
            })?;
        }

        Ok(Variant {
            index: index
                .ok_or_else(|| syn::Error::new_spanned(input, "missing index attribute"))?,
            encoding,
            fields: parse_fields(&input.fields, false)?,
            ident: input.ident.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct IndexVariant {
    index: isize,
    ident: syn::Ident,
}

impl TryFrom<&syn::Variant> for IndexVariant {
    type Error = syn::Error;

    fn try_from(input: &syn::Variant) -> Result<Self, Self::Error> {
        let mut index = None;

        for attr in &input.attrs {
            if attr_index(&mut index, attr, false)? {
                continue;
            }
            if !attr.path().is_ident("cbor") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta_index(&mut index, &meta, false)? {
                    return Ok(());
                }
                Err(meta.error("unknown attribute"))
            })?;
        }

        Ok(IndexVariant {
            index: index
                .ok_or_else(|| syn::Error::new_spanned(input, "missing index attribute"))?,
            ident: input.ident.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FlatVariant {
    pub index: isize,
    pub fields: Vec<Field>,
    pub ident: syn::Ident,
}

impl TryFrom<&syn::Variant> for FlatVariant {
    type Error = syn::Error;

    fn try_from(input: &syn::Variant) -> Result<Self, Self::Error> {
        let mut index = None;

        for attr in &input.attrs {
            if attr_index(&mut index, attr, true)? {
                continue;
            }
            if !attr.path().is_ident("cbor") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta_index(&mut index, &meta, true)? {
                    return Ok(());
                }
                Err(meta.error("unknown attribute"))
            })?;
        }

        Ok(FlatVariant {
            index: index
                .ok_or_else(|| syn::Error::new_spanned(input, "missing index attribute"))?,
            fields: parse_fields(&input.fields, true)?,
            ident: input.ident.clone(),
        })
    }
}
