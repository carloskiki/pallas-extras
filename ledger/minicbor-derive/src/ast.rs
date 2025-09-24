use enumeration::Enum;
use quote::{format_ident, quote, ToTokens};
use structure::Struct;

mod enumeration;
mod structure;

/// A container that derives `Encode`, `Decode`, or `CborLen`.
pub struct Container {
    pub tag: Option<usize>,
    pub bounds: Bounds,
    pub data: Data,
    pub original: syn::DeriveInput,
}

impl Container {
    pub fn encode(self) -> proc_macro2::TokenStream {
        let Container {
            tag,
            bounds:
                Bounds {
                    bound,
                    encode_bound,
                    context_bound,
                    ..
                },
            data,
            original:
                syn::DeriveInput {
                    ident,
                    mut generics,
                    ..
                },
        } = self;
        let (_, ty_generics, _) = generics.split_for_impl();
        let ty_generics = quote! { #ty_generics };
        
        generics.params.push(syn::parse_quote! { C });
        let (impl_generics, _, where_clause) = generics.split_for_impl();
        
        let tag = tag.map(|t| quote! { e.tag(#t)?; });

        let procedure = data.encode();

        quote! {
            impl #impl_generics ::minicbor::Encode<C> for #ident #ty_generics
                #where_clause
                #(#bound,)*
                #(#encode_bound,)*
                #(#context_bound,)*
            {
                fn encode<W: ::minicbor::encode::Write>(
                    &self,
                    e: &mut ::minicbor::encode::Encoder<W>,
                    ctx: &mut C
                ) -> Result<(), ::minicbor::encode::Error<W::Error>> {
                    #tag
                    #procedure
                    Ok(())
                }
            }
        }
    }

    pub fn decode(self) -> proc_macro2::TokenStream {
        todo!()
    }

    pub fn len(self) -> proc_macro2::TokenStream {
        todo!()
    }
}

impl TryFrom<syn::DeriveInput> for Container {
    type Error = syn::Error;

    fn try_from(input: syn::DeriveInput) -> Result<Self, Self::Error> {
        let mut tag = None;
        let mut bounds = Bounds::default();
        let mut encoding = None;

        for attr in &input.attrs {
            if !attr.path().is_ident("cbor") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if bounds.update(&meta)?
                    || tag_update(&mut tag, &meta)?
                    || encoding_update(&mut encoding, &meta)?
                {
                    return Ok(());
                }

                if !meta.path.is_ident("index_only") && !meta.path.is_ident("flat") {
                    return Err(meta.error("unknown attribute"));
                }
                Ok(())
            })?;
        }
        
        let data = match input.data {
            syn::Data::Struct(ref data) => Data::Struct(Struct::parse(data, encoding)?),
            syn::Data::Enum(ref data) => Data::Enum(input.attrs.iter().try_fold(
                Enum::normal(data, encoding)?,
                |mut enu, attr| {
                    if !attr.path().is_ident("cbor") {
                        return Ok::<_, syn::Error>(enu);
                    }
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("index_only") {
                            match enu {
                                Enum::Normal { .. } => {
                                    enu = Enum::index(data)?;
                                }
                                Enum::Index { .. } => {
                                    return Err(meta.error("duplicate `index_only` attribute"));
                                }
                                Enum::Flat { .. } => {
                                    return Err(meta.error(
                                        "cannot combine `flat` and `index_only` attributes",
                                    ));
                                }
                            }
                        } else if meta.path.is_ident("flat") {
                            match enu {
                                Enum::Normal { .. } => {
                                    enu = Enum::flat(data)?;
                                }
                                Enum::Index { .. } => {
                                    return Err(meta.error(
                                        "cannot combine `flat` and `index_only` attributes",
                                    ));
                                }
                                Enum::Flat { .. } => {
                                    return Err(meta.error("duplicate `flat` attribute"));
                                }
                            }
                        }
                        Ok(())
                    })?;
                    Ok(enu)
                },
            )?),
            syn::Data::Union(_) => {
                return Err(syn::Error::new_spanned(
                    input.ident,
                    "unions are not supported",
                ));
            }
        };

        Ok(Container {
            tag,
            bounds,
            data,
            original: input,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Data {
    Struct(Struct),
    Enum(Enum),
}

impl Data {
    pub fn encode(self) -> proc_macro2::TokenStream {
        match self {
            Data::Struct(s) => s.encode(true),
            Data::Enum(e) => e.encode(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub index: isize,
    pub skip: bool,
    pub default: bool,
    pub tag: Option<usize>,
    pub decode_with: Option<syn::Path>,
    pub encode_with: Option<syn::Path>,
    pub len_with: Option<syn::Path>,
    pub with: Option<syn::Path>,
    pub member: syn::Member,
}

impl Field {
    fn parse(value: &syn::Field, member: syn::Member, flat: bool) -> syn::Result<Self> {
        let syn::Field { attrs, .. } = value;
        let mut index = None;
        let mut skip = false;
        let mut default = false;
        let mut tag = None;
        let mut decode_with = None;
        let mut encode_with = None;
        let mut len_with = None;
        let mut with = None;

        for attr in attrs {
            if attr_index(&mut index, attr, flat)? {
                continue;
            }
            if !attr.path().is_ident("cbor") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta_index(&mut index, &meta, flat)? {
                    return Ok(());
                } else if meta.path.is_ident("skip") {
                    if skip {
                        return Err(meta.error("duplicate skip attribute"));
                    }
                    skip = true;
                } else if meta.path.is_ident("default") {
                    if default {
                        return Err(meta.error("duplicate default attribute"));
                    }
                    default = true;
                } else if meta.path.is_ident("tag") {
                    if tag.is_some() {
                        return Err(meta.error("duplicate tag attribute"));
                    }
                    let val: syn::LitInt = meta.value()?.parse()?;
                    tag = Some(val.base10_parse::<usize>()?);
                } else if meta.path.is_ident("decode_with") {
                    if decode_with.is_some() {
                        return Err(meta.error("duplicate `decode_with` attribute"));
                    }
                    let val: syn::LitStr = meta.value()?.parse()?;
                    decode_with = Some(val.parse::<syn::Path>()?);
                } else if meta.path.is_ident("encode_with") {
                    if encode_with.is_some() {
                        return Err(meta.error("duplicate `encode_with` attribute"));
                    }
                    let val: syn::LitStr = meta.value()?.parse()?;
                    encode_with = Some(val.parse::<syn::Path>()?);
                } else if meta.path.is_ident("len_with") {
                    if len_with.is_some() {
                        return Err(meta.error("duplicate `len_with` attribute"));
                    }
                    let val: syn::LitStr = meta.value()?.parse()?;
                    len_with = Some(val.parse::<syn::Path>()?);
                } else if meta.path.is_ident("with") {
                    if with.is_some() {
                        return Err(meta.error("duplicate `with` attribute"));
                    }
                    let val: syn::LitStr = meta.value()?.parse()?;
                    with = Some(val.parse::<syn::Path>()?);
                } else {
                    return Err(meta.error("unknown attribute"));
                }
                Ok(())
            })?;
        }
        
        let Some(index) = index else {
            return Err(syn::Error::new_spanned(
                value,
                "missing required field attribute: `#[cbor(n = <index>)]`",
            ));
        };
        Ok(Field {
            index,
            skip,
            default,
            tag,
            decode_with,
            encode_with,
            len_with,
            with,
            member,
        })
    }

    fn encode(self, use_self: bool) -> proc_macro2::TokenStream {
        let Field {
            skip,
            tag,
            encode_with,
            with,
            member,
            ..
        } = self;
        if skip {
            return proc_macro2::TokenStream::new();
        }
        let tag = tag.map(|t| quote! { e.tag(#t)?; });
        let member = if use_self {
            quote! { self.#member }
        } else {
            format_ident!("_{}", member).to_token_stream()
        };
        let encode_procedure = if let Some(encode_with) = encode_with {
            quote! { #encode_with(&#member, e, ctx)?; }
        } else if let Some(with) = with {
            quote! { #with::encode(&#member, e, ctx)?; }
        } else {
            quote! { e.encode_with(&#member, ctx)?; }
        };
        quote! {
            #tag
            #encode_procedure
        }
    }
}

pub fn parse_fields(fields: &syn::Fields, flat: bool) -> syn::Result<Vec<Field>> {
    fields
        .iter()
        .zip(fields.members())
        .map(|(field, member)| Field::parse(field, member, flat))
        .collect()
}

#[derive(Debug, Clone, Default)]
pub struct Bounds {
    pub bound: Vec<syn::WherePredicate>,
    pub decode_bound: Vec<syn::WherePredicate>,
    pub encode_bound: Vec<syn::WherePredicate>,
    pub len_bound: Vec<syn::WherePredicate>,
    pub context_bound: Vec<syn::TypeParamBound>,
}

impl Bounds {
    pub fn update(&mut self, meta: &syn::meta::ParseNestedMeta) -> syn::Result<bool> {
        if meta.path.is_ident("bound") {
            let val: syn::LitStr = meta.value()?.parse()?;
            self.bound.push(val.parse()?);
        } else if meta.path.is_ident("decode_bound") {
            let val: syn::LitStr = meta.value()?.parse()?;
            self.decode_bound.push(val.parse()?);
        } else if meta.path.is_ident("encode_bound") {
            let val: syn::LitStr = meta.value()?.parse()?;
            self.encode_bound.push(val.parse()?);
        } else if meta.path.is_ident("len_bound") {
            let val: syn::LitStr = meta.value()?.parse()?;
            self.len_bound.push(val.parse()?);
        } else if meta.path.is_ident("context_bound") {
            let val: syn::LitStr = meta.value()?.parse()?;
            self.context_bound.push(val.parse()?);
        } else {
            return Ok(false);
        }
        Ok(true)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Encoding {
    #[default]
    Array,
    Map,
}

fn tag_update(tag: &mut Option<usize>, meta: &syn::meta::ParseNestedMeta) -> syn::Result<bool> {
    if meta.path.is_ident("tag") {
        if tag.is_some() {
            return Err(meta.error("Duplicate `tag` attribute"));
        }
        let val: syn::LitInt = meta.value()?.parse()?;
        *tag = Some(val.base10_parse()?);
        return Ok(true);
    }
    Ok(false)
}

fn attr_index(
    index: &mut Option<isize>,
    attribute: &syn::Attribute,
    flat: bool,
) -> syn::Result<bool> {
    if !attribute.path().is_ident("n") {
        return Ok(false);
    }

    if index.is_some() {
        return Err(syn::Error::new_spanned(
            attribute,
            "duplicate `n` attribute",
        ));
    } else if flat {
        return Err(syn::Error::new_spanned(
            attribute,
            "`#[n(<index>)]` is not allowed in `flat` enums",
        ));
    }
    let val = attribute.parse_args::<syn::LitInt>()?;
    *index = Some(val.base10_parse::<isize>()?);

    Ok(true)
}

fn meta_index(
    index: &mut Option<isize>,
    meta: &syn::meta::ParseNestedMeta,
    flat: bool,
) -> syn::Result<bool> {
    if !meta.path.is_ident("n") {
        return Ok(false);
    }

    if index.is_some() {
        return Err(meta.error("Duplicate index attribute"));
    } else if flat {
        return Err(meta.error("`n` attribute is not allowed in `flat` enums"));
    }
    let val: syn::LitInt = meta.value()?.parse()?;
    *index = Some(val.base10_parse()?);

    Ok(true)
}

fn encoding_update(
    encoding: &mut Option<Encoding>,
    meta: &syn::meta::ParseNestedMeta,
) -> syn::Result<bool> {
    if meta.path.is_ident("map") {
        if encoding.is_some() {
            return Err(meta.error("duplicate encoding attribute"));
        }
        *encoding = Some(Encoding::Map);
        return Ok(true);
    } else if meta.path.is_ident("array") {
        if encoding.is_some() {
            return Err(meta.error("duplicate encoding attribute"));
        }
        *encoding = Some(Encoding::Array);
        return Ok(true);
    }
    Ok(false)
}
