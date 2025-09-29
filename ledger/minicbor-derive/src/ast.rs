use enumeration::Enum;
use quote::{ToTokens, format_ident, quote};
use structure::Struct;
use syn::Ident;

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
    pub fn decode(self) -> proc_macro2::TokenStream {
        let Container {
            tag,
            bounds:
                Bounds {
                    bound,
                    decode_bound,
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
        let (impl_generics, ty_generics, where_clause) = context_generics(&mut generics);

        let tag = tag
            .map_or(quote! { true }, |t| {
                let t = t as u64;
                quote! { tag.as_u64() == #t }
            });

        let procedure = data.decode();

        quote! {
            impl #impl_generics ::minicbor::Decode<'_, __C> for #ident #ty_generics
                #where_clause
                #(#bound,)*
                #(#decode_bound,)*
                #(__C: #context_bound,)*
            {
                fn decode(
                    d: &mut ::minicbor::decode::Decoder<'_>,
                    ctx: &mut __C
                ) -> Result<Self, ::minicbor::decode::Error> {
                    let tag = d.tag()?;
                    if #tag {
                        #procedure
                    } else {
                        Err(::minicbor::decode::Error::tag_mismatch(tag).at(d.position()))
                    }
                }
            }
        }
    }

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
        let (impl_generics, ty_generics, where_clause) = context_generics(&mut generics);

        let tag = tag.map(|t| quote! { e.tag(#t)?; });

        let procedure = data.encode();

        quote! {
            impl #impl_generics ::minicbor::Encode<__C> for #ident #ty_generics
                #where_clause
                #(#bound,)*
                #(#encode_bound,)*
                #(__C: #context_bound,)*
            {
                fn encode<W: ::minicbor::encode::Write>(
                    &self,
                    e: &mut ::minicbor::encode::Encoder<W>,
                    ctx: &mut __C
                ) -> Result<(), ::minicbor::encode::Error<W::Error>> {
                    #tag
                    #procedure
                    Ok(())
                }
            }
        }
    }

    pub fn len(self) -> proc_macro2::TokenStream {
        let Container {
            tag,
            bounds:
                Bounds {
                    bound,
                    len_bound,
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
        let (impl_generics, ty_generics, where_clause) = context_generics(&mut generics);

        let tag = tag.map(|t| quote! { #t.cbor_len(ctx) + });

        let procedure = data.len();

        quote! {
            impl #impl_generics ::minicbor::CborLen<__C> for #ident #ty_generics
                #where_clause
                #(#bound,)*
                #(#len_bound,)*
                #(__C: #context_bound,)*
            {
                fn cbor_len(
                    &self,
                    ctx: &mut __C
                ) -> usize {
                    #tag #procedure
                }
            }
        }
    }
}

fn context_generics(
    generics: &mut syn::Generics,
) -> (
    syn::ImplGenerics<'_>,
    proc_macro2::TokenStream,
    Option<&syn::WhereClause>,
) {
    let (_, ty_generics, _) = generics.split_for_impl();
    let ty_generics = quote! { #ty_generics };
    generics.params.push(syn::parse_quote! { __C });
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    (impl_generics, ty_generics, where_clause)
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
    pub fn decode(self) -> proc_macro2::TokenStream {
        match self {
            Data::Struct(s) => s.decode(None),
            Data::Enum(e) => e.decode(),
        }
    }

    pub fn encode(self) -> proc_macro2::TokenStream {
        match self {
            Data::Struct(s) => s.encode(true),
            Data::Enum(e) => e.encode(),
        }
    }

    pub fn len(self) -> proc_macro2::TokenStream {
        match self {
            Data::Struct(s) => s.len(true),
            Data::Enum(e) => e.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub index: isize,
    pub optional: bool,
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
        let mut optional = false;
        let mut index = None;
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
                } else if meta.path.is_ident("optional") {
                    if optional {
                        return Err(meta.error("duplicate optional attribute"));
                    }
                    optional = true;
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
            optional,
            tag,
            decode_with,
            encode_with,
            len_with,
            with,
            member,
        })
    }

    fn decode(self) -> proc_macro2::TokenStream {
        let Field {
            tag,
            decode_with,
            with,
            ..
        } = self;
        let tag = tag.map_or(quote! {}, |t| {
            let t = t as u64;
            quote! { 
                let tag = d.tag()?;
                if !(tag.as_u64() == #t) {
                    return Err(::minicbor::decode::Error::tag_mismatch(tag).at(d.position()));
                }
            }
        });

        let decode_procedure = if let Some(decode_with) = decode_with {
            quote! { #decode_with(d, ctx)? }
        } else if let Some(with) = with {
            quote! { #with::decode(d, ctx)? }
        } else {
            quote! { ::minicbor::Decode::decode(d, ctx)? }
        };

        quote! {
            #tag
            #decode_procedure
        }
    }

    fn encode(self, use_self: bool) -> proc_macro2::TokenStream {
        let Field {
            tag,
            encode_with,
            with,
            member,
            ..
        } = self;
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

    fn len(self, use_self: bool) -> proc_macro2::TokenStream {
        let Field {
            tag,
            len_with,
            with,
            member,
            ..
        } = self;
        let tag = tag.map(|t| quote! { #t.cbor_len(ctx) + });
        let member = if use_self {
            quote! { self.#member }
        } else {
            format_ident!("_{}", member).to_token_stream()
        };
        let len_procedure = if let Some(len_with) = len_with {
            quote! { #len_with(&#member, ctx) }
        } else if let Some(with) = with {
            quote! { #with::cbor_len(&#member, ctx) }
        } else {
            quote! { ::minicbor::CborLen::cbor_len(&#member, ctx) }
        };
        quote! {
            #tag #len_procedure
        }
    }

    fn variable(&self) -> Ident {
        let member = &self.member;
        format_ident!("_{}", member)
    }
}

pub fn parse_fields(fields: &syn::Fields, flat: bool) -> syn::Result<Vec<Field>> {
    fields
        .iter()
        .zip(fields.members())
        .map(|(field, member)| Field::parse(field, member, flat))
        .collect()
}

pub fn struct_pattern(name: Ident, fields: &[Field]) -> proc_macro2::TokenStream {
    let members = fields
        .iter()
        .map(|f| {
            let variable = f.variable();
            let member = &f.member;
            quote! { #member: #variable }
        })
        .collect::<Vec<_>>();
    quote! { Self::#name { #(#members),* } }
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
