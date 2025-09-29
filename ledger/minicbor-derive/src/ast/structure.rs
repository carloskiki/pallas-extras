use proc_macro2::Span;
use quote::quote;
use syn::Ident;

use crate::ast::struct_pattern;

use super::{Encoding, Field, parse_fields};

#[derive(Debug, Clone)]
pub struct Struct {
    pub encoding: Encoding,
    pub fields: Vec<Field>,
}

impl Struct {
    pub fn parse(input: &syn::DataStruct, encoding: Option<Encoding>) -> syn::Result<Self> {
        let encoding = encoding.unwrap_or_default();
        let mut fields = parse_fields(&input.fields, false)?;
        fields.sort_by_key(|f| f.index);
        if matches!(encoding, Encoding::Array) {
            if fields.first().is_some_and(|f| f.index < 0) {
                return Err(syn::Error::new_spanned(
                    &input.fields,
                    "negative field indices are not allowed in array encoding",
                ));
            }
            if fields.iter().any(|f| f.optional) {
                return Err(syn::Error::new_spanned(
                    &input.fields,
                    "optional fields are not allowed in array encoding",
                ));
            }
        }

        Ok(Struct { encoding, fields })
    }

    pub fn decode(self, variant: Option<Ident>) -> proc_macro2::TokenStream {
        let struct_pattern = struct_pattern(
            variant
                .clone()
                .unwrap_or(Ident::new("Self", Span::call_site())),
            &self.fields,
        );
        let constructor = if variant.is_some() {
            quote! { Self::#struct_pattern }
        } else {
            quote! { #struct_pattern }
        };
        if matches!(self.encoding, Encoding::Array) {
            let array_len = self.fields.last().map_or(0, |f| f.index + 1) as u64;
            let mut fields = self.fields.into_iter().peekable();
            let field_procedures = (0..array_len).map(|i| {
                let field = fields
                    .peek()
                    .expect("the last index is at most the last field");
                if field.index as u64 == i {
                    let field = fields.next().expect("peeked");
                    let variable = field.variable();
                    let procedure = field.decode();
                    quote! {
                        let #variable = { #procedure };
                    }
                } else {
                    quote! {
                        d.skip()?;
                    }
                }
            });

            quote! {
                let array_len = d.array()?;
                
                if array_len.is_some_and(|l| l < #array_len) {
                    return Err(::minicbor::decode::Error::message(
                        concat!("array length to small, needs to be at least ", stringify!(#array_len))
                    ));
                }

                #(#field_procedures)*
                for _ in #array_len..array_len {
                    d.skip()?;
                }
                #constructor
            }
        } else {
            let field_variables = self
                .fields
                .iter()
                .map(|f| {
                    let variable = f.variable();
                    quote! {
                        let mut #variable = None;
                    }
                })
                .collect::<Vec<_>>();
            let field_checks = self.fields.iter().map(|f| {
                let variable = f.variable();
                let index = f.index;
                let field_name = f.member.clone();
                if f.optional {
                    quote! {
                        let #variable = #variable.unwrap_or_default();
                    }
                } else {
                    quote! {
                        let #variable = #variable.ok_or_else(|| ::minicbor::decode::Error::message(
                            concat!("missing field ", stringify!(#field_name), " with index " stringify!(#index))
                        ))?;
                    }
                }
            }).collect::<Vec<_>>();

            let field_procedures = self.fields.into_iter().map(|f| {
                let index = f.index as i64;
                let variable = f.variable();
                let field_proc = f.decode();
                quote! {
                    #index => { #variable = Some({ #field_proc }) };
                }
            });

            quote! {
                #(#field_variables)*
                let map_len = d.map()?;
                for _ in 0..map_len {
                    match d.i64()? {
                        #(#field_procedures)*
                        _ => { d.skip()?; }
                    }
                }
                #(#field_checks)*
                #constructor
            }
        }
    }

    pub fn encode(self, use_self: bool) -> proc_macro2::TokenStream {
        if matches!(self.encoding, Encoding::Array) {
            let array_len = self.fields.last().map_or(0, |f| f.index + 1) as u64;
            let mut fields = self.fields.into_iter().peekable();
            let field_procedures = (0..array_len).map(|i| {
                let field = fields
                    .peek()
                    .expect("the last index matches the last field");
                if field.index as u64 == i {
                    let field = fields.next().expect("peeked");
                    field.encode(use_self)
                } else {
                    quote! {
                        e.null()?;
                    }
                }
            });
            quote! {
                e.array(#array_len)?;
                #(#field_procedures)*
            }
        } else {
            let map_len = self.fields.len() as u64;
            let field_procedures = self.fields.into_iter().map(|f| {
                let index = f.index as i64;
                let field_proc = f.encode(use_self);
                quote! {
                    e.i64(#index as i64)?;
                    #field_proc
                }
            });
            quote! {
                e.map(#map_len)?;
                #(#field_procedures)*
            }
        }
    }

    pub fn len(self, use_self: bool) -> proc_macro2::TokenStream {
        if matches!(self.encoding, Encoding::Array) {
            let array_len = self.fields.last().map_or(0, |f| f.index + 1) as u64;
            let mut fields = self.fields.into_iter().peekable();
            let field_lens = (0..array_len).map(|i| {
                let field = fields
                    .peek()
                    .expect("the last index matches the last field");
                if field.index as u64 == i {
                    let field = fields.next().expect("peeked");
                    field.len(use_self)
                } else {
                    quote! { 1 } // null
                }
            });

            quote! {
                 #array_len.cbor_len(ctx) #( + #field_lens )*
            }
        } else {
            let field_lens = self
                .fields
                .into_iter()
                .map(|f| {
                    let field_index = f.index;
                    let field_len = f.len(use_self);
                    Some(quote! {
                        #field_index.cbor_len(ctx) + #field_len
                    })
                })
                .collect::<Vec<_>>();
            let map_len = field_lens.len() as u64;
            quote! {
                #map_len.cbor_len(ctx) + #( #field_lens )+*
            }
        }
    }
}
