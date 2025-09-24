use quote::quote;

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
        if matches!(encoding, Encoding::Array) && fields.first().is_some_and(|f| f.index < 0) {
            return Err(syn::Error::new_spanned(
                &input.fields,
                "negative field indices are not allowed in array encoding",
            ));
        }

        Ok(Struct { encoding, fields })
    }

    pub fn encode(mut self, use_self: bool) -> proc_macro2::TokenStream {
        self.fields.retain(|f| !f.skip);
        
        if matches!(self.encoding, Encoding::Array) {
            let array_len = self.fields.last().map_or(0, |f| f.index + 1) as u64;
            let mut fields = self.fields.into_iter().peekable();
            let field_procedures = (0..array_len).map(|i| {
                let field = fields.peek().expect("the last index matches the last field");
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
}
