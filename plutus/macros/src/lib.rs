extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn builtin(_args: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig:
            syn::Signature {
                ident,
                inputs,
                output,
                constness,
                asyncness,
                unsafety,
                abi,
                fn_token: _,
                generics,
                paren_token: _,
                variadic,
            },
        block,
    } = parse_macro_input!(item as ItemFn);
    if !generics.params.is_empty() {
        return syn::Error::new_spanned(
            generics,
            "#[builtin] functions cannot have generic parameters",
        )
        .to_compile_error()
        .into();
    }
    if variadic.is_some() {
        return syn::Error::new_spanned(variadic, "#[builtin] functions cannot be variadic")
            .to_compile_error()
            .into();
    }
    let syn::ReturnType::Type(_, output_type) = output else {
        return syn::Error::new_spanned(
            output,
            "#[builtin] functions must have an explicit return type",
        )
        .to_compile_error()
        .into();
    };
    let input_vars = inputs
        .into_iter()
        .enumerate()
        .map(|(i, arg)| match arg {
            syn::FnArg::Receiver(_) => {
                syn::Error::new_spanned(
                    arg,
                    "#[builtin] functions cannot have a `self` parameter",
                )
                .to_compile_error()
            }
            syn::FnArg::Typed(pat_type) => {
                let syn::PatType {
                    attrs,
                    pat,
                    colon_token: _,
                    ty,
                } = pat_type;
                if !attrs.is_empty() {
                    return syn::Error::new_spanned(
                        &attrs[0],
                        "#[builtin] function parameters cannot have attributes",
                    )
                    .to_compile_error();
                }
                if ty == syn::parse_quote! { ValueIndex } {
                    return quote::quote! {
                        let #pat = ValueIndex(#i as u32);
                    };
                }
                let (konst_pat, other_pat) = match unwrap_type(*ty.clone()) {
                    Ok(res) => res,
                    Err(err) => return err.to_compile_error(),
                };
                let other_pat = other_pat.map(|rest| quote::quote! {
                        && let #rest
                });
                
                quote::quote! {
                    let #pat: #ty = {
                        let Value::Constant(constant_index) = &args[#i] else {
                            return None;
                        };
                        if let #konst_pat = std::mem::take(&mut constants[constant_index.0 as usize])
                            #other_pat
                        {
                            constant
                        } else {
                            return None;
                        }
                    };
                }
            }
        })
        .collect::<Vec<_>>();

    let wrap_result = if output_type == syn::parse_quote! { ValueIndex } {
        quote::quote! {
            Some(args.swap_remove(result.0 as usize))
        }
    } else {
        let wrapped = wrap_type(*output_type).unwrap_or_else(|err| err.to_compile_error());
        quote::quote! {
            let Value::Constant(const_index) = args[0] else {
                panic!("Invariant violation: expected the first argument to builtin to be a constant");
            };

            constants[const_index.0 as usize] = #wrapped;
            Some(Value::Constant(const_index))
        }
    };

    quote::quote! {
        #(#attrs)* #vis #constness #asyncness #unsafety #abi fn #ident
        (mut args: Vec<crate::cek::Value>, constants: &mut [crate::constant::Constant]) -> Option<crate::cek::Value> {
            use crate::cek::Value;
            use crate::constant::Constant;
            use crate::ValueIndex;
            #(#input_vars)*

            let result = (|| #block)();
            #wrap_result
        }
    }
    .into()
}

fn unwrap_type(
    ty: syn::Type,
) -> syn::Result<(proc_macro2::TokenStream, Option<proc_macro2::TokenStream>)> {
    match ty.clone() {
        syn::Type::Path(mut type_path) => {
            let syn::PathSegment { ident, arguments } =
                type_path.path.segments.pop().unwrap().into_value();
            Ok(if ident == "Constant" {
                (
                    quote::quote! {
                        constant
                    },
                    None,
                )
            } else if ident == "Integer" {
                (
                    quote::quote! {
                        Constant::Integer(constant)
                    },
                    None,
                )
            } else if ident == "String" {
                (
                    quote::quote! {
                        Constant::String(constant)
                    },
                    None,
                )
            } else if ident == "bool" {
                (
                    quote::quote! {
                        Constant::Boolean(constant)
                    },
                    None,
                )
            } else if ident == "Data" {
                (
                    quote::quote! {
                        Constant::Data(constant)
                    },
                    None,
                )
            } else if ident == "G1Projective" {
                (
                    quote::quote! {
                        Constant::BlsG1Element(constant)
                    },
                    None,
                )
            } else if ident == "G2Projective" {
                (
                    quote::quote! {
                        Constant::BlsG2Element(constant)
                    },
                    None,
                )
            } else if ident == "Vec" {
                let elem_ty = type_arg(arguments)?;
                if elem_ty == syn::parse_quote! { u8 } {
                    (
                        quote::quote! {
                            Constant::Bytes(constant)
                        },
                        None,
                    )
                } else {
                    let (konst_pat, other_pat) = unwrap_type(elem_ty)?;
                    let other_pat = other_pat.map(|rest| quote::quote! {
                        && let #rest
                    });
                    
                    (
                        quote::quote! {
                            Constant::List(list)
                        },
                        Some(quote::quote! {
                            Some(constant) = {
                                list.into_iter().map(|item| {
                                    if let #konst_pat = item
                                        #other_pat
                                    {
                                        Some(constant)
                                    } else {
                                        None
                                    }
                                }).collect::<Option<Vec<_>>>()
                            }
                        }),
                    )
                }
            } else if ident == "Box" {
                let syn::Type::Slice(syn::TypeSlice {
                    elem,
                    ..
                }) = type_arg(arguments)? else {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "#[builtin] function parameters of `Box` type must be slices",
                    ));
                };
                let (konst_pat, other_pat) = unwrap_type(*elem)?;
                let other_pat = other_pat.map(|rest| quote::quote! {
                        && let #rest
                });
                
                (
                    quote::quote! {
                        Constant::Array(array)
                    },
                    Some(quote::quote! {
                        Some(constant) = {
                            array.into_iter().map(|item| {
                                if let #konst_pat = item
                                    #other_pat
                                {
                                    Some(constant)
                                } else {
                                    None
                                }
                            }).collect::<Option<Vec<_>>>()
                        }
                    }),
                )

                
            } else {
                return Err(syn::Error::new_spanned(
                    ty,
                    format!(
                        "#[builtin] function parameters of type `{}` are not supported",
                        ident
                    ),
                ));
            })
        }
        syn::Type::Tuple(type_tuple) => {
            if type_tuple.elems.is_empty() {
                Ok((
                    quote::quote! {
                        Constant::Unit
                    },
                    Some(quote::quote! {
                        constant = ()
                    })
                ))
            } else if type_tuple.elems.len() == 2 {
                let mut elems = type_tuple.elems.into_iter();
                let first_ty = elems.next().unwrap();
                let second_ty = elems.next().unwrap();
                let (first_pat, first_other) = unwrap_type(first_ty)?;
                let (second_pat, second_other) = unwrap_type(second_ty)?;
                let first_other = first_other.map(|rest| quote::quote! {
                        && let #rest
                });
                let second_other = second_other.map(|rest| quote::quote! {
                        && let #rest
                });
                
                Ok((
                    quote::quote! {
                        Constant::Pair(pair)
                    },
                    Some(quote::quote! {
                        Ok(constant) = (|| {
                            let (first, second) = *pair;
                            let first = if let #first_pat = first
                                #first_other
                            {
                                constant
                            } else {
                                return Err(());
                            };
                            let second = if let #second_pat = second
                                #second_other
                            {
                                constant
                            } else {
                                return Err(());
                            };
                            Ok((first, second))
                        })()
                    }),
                ))
            } else {
                Err(syn::Error::new_spanned(
                    ty,
                    "#[builtin] function parameters can only be unit `()` or pairs `(T1, T2)`",
                ))
            }
        }
        _ => Err(syn::Error::new_spanned(
            ty,
            "#[builtin] function parameters must be a simple type",
        )),
    }
}

fn wrap_type(ty: syn::Type) -> syn::Result<proc_macro2::TokenStream> {
    match ty.clone() {
        syn::Type::Path(mut type_path) => {
            let syn::PathSegment { ident, arguments } =
                type_path.path.segments.pop().unwrap().into_value();
            Ok(if ident == "Constant" {
                quote::quote! {
                    result
                }
            } else if ident == "Integer" {
                quote::quote! {
                    Constant::Integer(result)
                }
            } else if ident == "String" {
                quote::quote! {
                    Constant::String(result)
                }
            } else if ident == "bool" {
                quote::quote! {
                    Constant::Boolean(result)
                }
            } else if ident == "Data" {
                quote::quote! {
                    Constant::Data(result)
                }
            } else if ident == "G1Projective" {
                quote::quote! {
                    Constant::BlsG1Element(Box::new(result))
                }
            } else if ident == "G2Projective" {
                quote::quote! {
                    Constant::BlsG2Element(Box::new(result))
                }
            } else if ident == "Vec" {
                let elem_ty = type_arg(arguments)?;
                if elem_ty == syn::parse_quote! { u8 } {
                    quote::quote! {
                        Constant::Bytes(result)
                    }
                } else {
                    let wrapped_elem = wrap_type(elem_ty)?;
                    quote::quote! {{
                        let list = result.into_iter().map(|result| {
                            #wrapped_elem
                        }).collect::<Vec<_>>();
                        Constant::List(list)
                    }}
                }
            } else if ident == "Box" {
                let syn::Type::Slice(syn::TypeSlice {
                    elem,
                    ..
                }) = type_arg(arguments)? else {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "#[builtin] function return types of `Box` type must be slices",
                    ));
                };
                let wrapped_elem = wrap_type(*elem)?;
                quote::quote! {{
                    let array = result.into_iter().map(|result| {
                        #wrapped_elem
                    }).collect::<Box<[_]>>();
                    Constant::Array(array)
                }}
            } else if ident == "Option" {
                let elem_ty = type_arg(arguments)?;
                let wrapped_elem = wrap_type(elem_ty)?;
                quote::quote! {{
                    let result = result?;
                    #wrapped_elem
                }}
            } else {
                return Err(syn::Error::new_spanned(
                    ty,
                    format!(
                        "#[builtin] function return types of `{}` are not supported",
                        ident
                    ),
                ));
            })
        }

        syn::Type::Tuple(type_tuple) => {
            if type_tuple.elems.is_empty() {
                Ok(quote::quote! {
                    Constant::Unit
                })
            } else if type_tuple.elems.len() == 2 {
                let mut elems = type_tuple.elems.into_iter();
                let first_ty = elems.next().unwrap();
                let second_ty = elems.next().unwrap();
                let wrapped_first = wrap_type(first_ty)?;
                let wrapped_second = wrap_type(second_ty)?;
                Ok(quote::quote! {
                    let (first, second) = result;
                    let first = {
                        let result = first;
                        #wrapped_first
                    };
                    let second = {
                        let result = second;
                        #wrapped_second
                    };
                    Constant::Pair(Box::new((first, second)))
                })
            } else {
                Err(syn::Error::new_spanned(
                    ty,
                    "#[builtin] function return types can only be unit `()` or pairs `(T1, T2)`",
                ))
            }
        }

        _ => Err(syn::Error::new_spanned(
            ty,
            "#[builtin] function return types must be a simple type",
        )),
    }
}

fn type_arg(args: syn::PathArguments) -> syn::Result<syn::Type> {
    let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { mut args, .. }) =
        args
    else {
        return Err(syn::Error::new_spanned(
            args,
            "Expected angle bracketed type arguments",
        ));
    };
    if args.len() != 1 {
        return Err(syn::Error::new_spanned(
            args,
            "Expected exactly one type argument",
        ));
    }
    let syn::GenericArgument::Type(ty) = args.pop().unwrap().into_value() else {
        return Err(syn::Error::new_spanned(
            args,
            "Expected type argument to be a type",
        ));
    };
    Ok(ty)
}
