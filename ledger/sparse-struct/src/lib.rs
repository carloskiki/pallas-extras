//! Derive a sparse struct out of its possible members.
//!
//! This utility allows the definition of a struct that can hold a sparse set of its possible
//! members. This solves the problem of having a struct with many optional fields, which takes up a
//! lot of memory even when most fields are unused.
//!
//! ## Example
//!
//! ```rust
//! # extern crate alloc;
//! 
//! #[derive(Debug, Clone, PartialEq, sparse_struct::SparseStruct)]
//! #[struct_derive(Debug, Clone, PartialEq)]
//! #[struct_name = "Person"]
//! enum Attribute {
//!     Age(u8),
//!     Name(String),
//!     Height(f32),
//!     Weight(f32),
//!     EyeColor(String),
//!     HairColor(String),
//!     SkinTone(String),
//!     BloodType(String),
//!     Allergies(Vec<String>),
//!     Medications(Vec<String>),
//!     PhoneNumber(String),
//!     Email(String),
//!     Address(String),
//!     Contact(String),
//! }
//!
//! let mut person = Person::from_iter([
//!     Attribute::Age(5),
//!     Attribute::Name("Alice".to_string()),
//!     Attribute::Age(20),
//! ]);
//!
//! assert_eq!(person.age(), Some(&20));
//! if let Some(age) = person.age_mut() {
//!     *age += 1;
//! }
//! assert_eq!(person.age(), Some(&21));
//!
//! person.remove_name();
//! assert!(person.name().is_none());
//!
//! person.insert(Attribute::Height(180.5));
//! assert_eq!(person.height(), Some(&180.5));
//!
//! assert_eq!(
//!     person.as_ref(),
//!     &[Attribute::Age(21), Attribute::Height(180.5)]
//! );
//! ```
//!
//! This generates a `Person` struct with a few helpful methods and trait implementations to access
//! attributes, and modify them.

use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Data, DataStruct, DataUnion, DeriveInput, Fields, Ident, parse_macro_input, spanned::Spanned,
    token::Struct,
};

#[proc_macro_derive(SparseStruct, attributes(struct_name, struct_derive))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(
    DeriveInput {
        vis,
        attrs,
        ident: enum_ident,
        generics,
        data,
    }: DeriveInput,
) -> syn::Result<TokenStream> {
    let syn::DataEnum { variants, .. } = match data {
        Data::Enum(e) => e,
        Data::Struct(DataStruct {
            struct_token: Struct { span },
            ..
        })
        | Data::Union(DataUnion {
            union_token: syn::token::Union { span },
            ..
        }) => {
            return Err(syn::Error::new(
                span,
                "`SparseStruct` can only be derived for `enum`s",
            ));
        }
    };
    if variants.len() > 64 {
        return Err(syn::Error::new(
            enum_ident.span(),
            "`SparseStruct` can only be derived for enums with up to 64 variants.",
        ));
    }

    let (methods, index_arms) = variants
        .iter()
        .enumerate()
        .map(|(i, variant)| {
            let span = variant.span();
            let field = match &variant.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0].ty,
                _ => {
                    return Err(syn::Error::new(
                        variant.span(),
                    "`SparseStruct` can only be derived for enums containing newtype variants (tuple variants with a single type).",
                    ));
                }
            };
            if let Some(err) = variant.attrs.iter().find_map(|attr| {
                ["struct_name", "struct_derive"].iter().find_map(|attr_name| {
                    attr.path().is_ident(attr_name).then_some(syn::Error::new(
                        span,
                        format!("`{attr_name}` should be specified on the enum, not on its variants."),
                    ))
                })
            }) {
                return Err(err);
            }

            let index_computation = quote! {
                let significant_bit = (1 << #i) as u64;
                if self.present & significant_bit == 0 {
                    return None;
                }
                let index = (self.present & (significant_bit - 1)).count_ones() as ::core::primitive::usize;
            };
            let variant_ident = &variant.ident;
            let get_fn_body = |ref_type| quote! {{
                #index_computation
                let #enum_ident::#variant_ident(data) = #ref_type self.data[index] else {
                    ::core::unreachable!(
                        "The variant should be present in the data array if the bit is set."
                    );
                };
                ::core::option::Option::Some(data)
            }};
            let fn_ref = get_fn_body(quote!(&));
            let fn_mut = get_fn_body(quote!(&mut));

            let fn_ident = Ident::new(
                &variant.ident.to_string().to_snake_case(),
                Span::call_site(),
            );
            let fn_ident_mut = Ident::new(&format!("{fn_ident}_mut"), Span::call_site());
            let set_ident = Ident::new(&format!("set_{fn_ident}"), Span::call_site());
            let remove_ident = Ident::new(&format!("remove_{fn_ident}"), Span::call_site());
            Ok((quote! {
                /// Returns a reference to the field if it is present.
                pub fn #fn_ident(&self) -> ::core::option::Option<&#field> #fn_ref
                /// Returns a mutable reference to the field if it is present.
                pub fn #fn_ident_mut(&mut self) -> ::core::option::Option<&mut #field> #fn_mut
                /// Removes the field from the set, returning it if it was present.
                pub fn #remove_ident(&mut self) -> ::core::option::Option<#field> {
                    #index_computation
                    let #enum_ident::#variant_ident(data) = self.data.remove(index) else {
                        ::core::unreachable!(
                            "The variant should be present in the data array if the bit is set."
                        );
                    };
                    self.present &= !significant_bit;
                    ::core::option::Option::Some(data)
                }
                /// Sets the field to the given value.
                ///
                /// Returns whether the value was newly inserted. That is:
                /// - `true` if the value was not present and has been added.
                /// - `false` if the value was already present and has been updated.
                pub fn #set_ident(&mut self, value: #field) -> bool {
                    let significant_bit = (1 << #i) as u64;
                    let variant = #enum_ident::#variant_ident(value);

                    if self.present & significant_bit != 0 {
                        // Update existing value.
                        let index = (self.present & (significant_bit - 1)).count_ones() as ::core::primitive::usize;
                        self.data[index] = variant;
                        false
                    } else {
                        // Insert new value.
                        let index = (self.present & (significant_bit - 1)).count_ones() as ::core::primitive::usize;
                        self.data.insert(index, variant);
                        self.present |= significant_bit;
                        true
                    }
                }
            }, quote! {
            #enum_ident::#variant_ident { .. } => #i,
        }))
        })
        .collect::<syn::Result<(TokenStream, TokenStream)>>()?;

    let mut struct_ident: Ident = format_ident!("{}Set", enum_ident);
    let mut struct_derives = quote! {};

    for attr in attrs {
        if attr.path().is_ident("struct_name") {
            match &attr.meta {
                syn::Meta::NameValue(syn::MetaNameValue {
                    value:
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }),
                    ..
                }) => struct_ident = format_ident!("{}", lit_str.value()),
                _ => {
                    return Err(syn::Error::new(
                        attr.span(),
                        "The `struct_name` attribute must be in the form `#[struct_name = \"Name\"]`.",
                    ));
                }
            }
        } else if attr.path().is_ident("struct_derive") {
            struct_derives = attr.parse_args()?;
        }
    }

    Ok(quote! {
        #[derive(#struct_derives)]
        #vis struct #struct_ident #generics {
            data: ::alloc::vec::Vec<#enum_ident #generics>,
            present: ::core::primitive::u64,
        }

        impl #generics #struct_ident #generics {
            #methods
        }

        impl #generics #struct_ident #generics {
            /// Inserts a new value into the set.
            ///
            /// Returns whether the value was newly inserted. That is:
            /// - `true` if the value was not present and has been added.
            /// - `false` if the value was already present and has been updated.
            pub fn insert(&mut self, value: #enum_ident #generics) -> ::core::primitive::bool {
                let variant_index = match &value {
                    #index_arms
                };
                let significant_bit = (1 << variant_index) as ::core::primitive::u64;
                if self.present & significant_bit != 0 {
                    // Update existing value.
                    let index = (self.present & (significant_bit - 1)).count_ones() as ::core::primitive::usize;
                    self.data[index] = value;
                    false
                } else {
                    // Insert new value.
                    let index = (self.present & (significant_bit - 1)).count_ones() as ::core::primitive::usize;
                    self.data.insert(index, value);
                    self.present |= significant_bit;
                    true
                }
            }

        }

        const _: () = {
        use ::core::{
            default::Default,
            convert::AsRef,
            iter::{FromIterator, IntoIterator, Iterator},
        };
        use ::alloc::vec::Vec;

        impl #generics Default for #struct_ident #generics {
            fn default() -> Self {
                Self {
                    data: Vec::new(),
                    present: 0,
                }
            }
        }

        /// Allows viewing the sparse struct as a slice of its present members.
        ///
        /// This slice is guaranteed to contain each present member exactly once, in the order of
        /// the enum variants definition.
        impl #generics AsRef<[#enum_ident #generics]> for #struct_ident #generics {
            fn as_ref(&self) -> &[#enum_ident #generics] {
                &self.data
            }
        }

        impl #generics FromIterator<#enum_ident #generics> for #struct_ident #generics {
            fn from_iter<T: IntoIterator<Item = #enum_ident #generics>>(iter: T) -> Self {
                iter.into_iter().fold(
                    #struct_ident {
                        data: Vec::new(),
                        present: 0,
                    },
                    |mut acc, item| {
                        acc.insert(item);
                        acc
                    },
                )
            }
        }
        };
    })
}
