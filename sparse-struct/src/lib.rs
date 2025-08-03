use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    Data, DataStruct, DataUnion, DeriveInput, Fields, Ident, parse_macro_input, spanned::Spanned,
    token::Struct,
};

#[proc_macro_derive(SparseStruct, attributes(struct_name))]
pub fn derive(input: TokenStream) -> TokenStream {
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
) -> syn::Result<proc_macro2::TokenStream> {
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

    let methods = variants
        .iter()
        .enumerate()
        .map(|(i, variant)| {
            let fields = match &variant.fields {
                Fields::Unit | Fields::Named(_) => {
                    return Err(syn::Error::new(
                        variant.span(),
                    "`SparseStruct` can only be derived for enums containing newtype variants (tuple variants with a single type).",
                    ));
                }
                Fields::Unnamed(fields) => fields,
            };
            if fields.unnamed.len() != 1 {
                return Err(syn::Error::new(
                    variant.span(),
                    "`SparseStruct` can only be derived for enums containing newtype variants (tuple variants with a single type).",
                ));
            }
            if variant.attrs.iter().any(|attr| {
                attr.path().is_ident("struct_name")
            }) {
                return Err(syn::Error::new(
                    variant.span(),
                    "The `struct_name` should be specified on the enum, not on its variants.",
                ));
            }

            let variant_ident = &variant.ident;
            let fn_body = |ref_type| quote! {{
                let significant_bit = 1 << #i as u64;
                if self.present & significant_bit as u64 == 0 {
                    return None;
                }
                let index = (self.present & (significant_bit - 1)).count_ones() as usize;
                let #enum_ident::#variant_ident(data) = #ref_type self.data[index] else {
                    unreachable!(
                        "The variant should be present in the data array if the bit is set."
                    );
                };
                Some(data)
            }};
            let fn_ref = fn_body(quote!(&));
            let fn_mut = fn_body(quote!(&mut));

            let fn_ident = Ident::new(
                &variant.ident.to_string().to_snake_case(),
                Span::call_site(),
            );
            let fn_ident_mut = Ident::new(&format!("{fn_ident}_mut"), Span::call_site());
            Ok(quote! {
                #[allow(unused_parens)]
                pub fn #fn_ident(&self) -> Option<&#fields> #fn_ref
                #[allow(unused_parens)]
                pub fn #fn_ident_mut(&mut self) -> Option<&mut #fields> #fn_mut
            })
        })
        .collect::<syn::Result<proc_macro2::TokenStream>>()?;

    let struct_ident: Ident = attrs
        .iter()
        .find_map(|attr| {
            if !attr.path().is_ident("struct_name") {
                return None;
            }
            Some(attr.parse_args())
        })
        .unwrap_or_else(|| Ok(format_ident!("{}Set", enum_ident)))?;

    Ok(quote! {
        #vis struct #struct_ident #generics {
            data: Box<[#enum_ident #generics]>,
            present: u64,
        }

        impl #generics AsRef<[#enum_ident #generics]> for #struct_ident #generics {
            fn as_ref(&self) -> &[#enum_ident #generics] {
                &self.data
            }
        }

        impl #generics #struct_ident #generics {
            #methods
        }
    })
}
