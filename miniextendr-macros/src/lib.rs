// miniextendr-macros procedural macros

struct ExtendrFunction {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub abi: Option<syn::Abi>,
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    pub output: syn::ReturnType,
}

impl syn::parse::Parse for ExtendrFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let itemfn: syn::ItemFn = input.parse()?;
        let signature: syn::Signature = itemfn.sig;

        Ok(Self {
            attrs: itemfn.attrs,
            vis: itemfn.vis,
            abi: signature.abi,
            ident: signature.ident,
            generics: signature.generics,
            inputs: signature.inputs,
            output: signature.output,
        })
    }
}

fn first_type_argument(seg: &syn::PathSegment) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
        for arg in ab.args.iter() {
            if let syn::GenericArgument::Type(ty) = arg {
                return Some(ty);
            }
        }
    }
    None
}

fn is_unit_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty())
}

fn is_sexp_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p
        .path
        .segments
        .last()
        .map(|s| s.ident == "SEXP")
        .unwrap_or(false))
}

/// Check if a type is `ExternalPtr<...>` which contains SEXP and is !Send.
/// This is used to automatically force main_thread for functions returning ExternalPtr.
fn is_external_ptr_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p
        .path
        .segments
        .last()
        .map(|s| s.ident == "ExternalPtr")
        .unwrap_or(false))
}

/// Check if visibility is exactly `pub` (not `pub(crate)`, `pub(super)`, or `pub(in path)`).
/// Used to determine whether to add `#' @export` to R wrappers.
fn is_pub_visibility(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

/// Result of coercion analysis for a type.
/// Contains the R native type to extract from SEXP and the target type to coerce to.
enum CoercionMapping {
    /// Scalar coercion: extract R native type, coerce to target
    Scalar {
        r_native: proc_macro2::TokenStream,
        target: proc_macro2::TokenStream,
    },
    /// Vec coercion: extract R native slice, coerce element-wise to Vec<target>
    Vec {
        r_native_elem: proc_macro2::TokenStream,
        target_elem: proc_macro2::TokenStream,
    },
}

/// Check if a function argument has a `#[miniextendr(coerce)]` attribute.
fn has_miniextendr_coerce_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("miniextendr") {
            // Parse the attribute arguments to check for "coerce"
            if let syn::Meta::List(list) = &attr.meta {
                if let Ok(nested) = list.parse_args::<syn::Ident>() {
                    return nested == "coerce";
                }
            }
        }
        false
    })
}

/// Get the coercion mapping for a type, if it needs coercion.
/// Returns None if the type is R-native (no coercion needed) or unknown.
fn get_coercion_mapping(ty: &syn::Type) -> Option<CoercionMapping> {
    match ty {
        syn::Type::Path(type_path) => {
            let seg = type_path.path.segments.last()?;
            let type_name = seg.ident.to_string();

            // Check for Vec<T> types
            if type_name == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        if let syn::Type::Path(inner_path) = inner_ty {
                            let inner_name = inner_path.path.segments.last()?.ident.to_string();
                            return match inner_name.as_str() {
                                // Vec<integer-like> from &[i32]
                                "u16" | "i16" | "i8" | "u32" | "u64" | "i64" => {
                                    let target_elem: proc_macro2::TokenStream =
                                        inner_name.parse().ok()?;
                                    Some(CoercionMapping::Vec {
                                        r_native_elem: quote::quote!(i32),
                                        target_elem,
                                    })
                                }
                                // Vec<f32> from &[f64]
                                "f32" => Some(CoercionMapping::Vec {
                                    r_native_elem: quote::quote!(f64),
                                    target_elem: quote::quote!(f32),
                                }),
                                _ => None,
                            };
                        }
                    }
                }
                return None;
            }

            // Check for scalar types
            match type_name.as_str() {
                // Integer-like types from i32
                "u16" | "i16" | "i8" | "u32" | "u64" | "i64" => {
                    let target: proc_macro2::TokenStream = type_name.parse().ok()?;
                    Some(CoercionMapping::Scalar {
                        r_native: quote::quote!(i32),
                        target,
                    })
                }
                // f32 from f64
                "f32" => Some(CoercionMapping::Scalar {
                    r_native: quote::quote!(f64),
                    target: quote::quote!(f32),
                }),
                // R-native types or unknown - no coercion
                _ => None,
            }
        }
        _ => None,
    }
}

#[proc_macro_attribute]
pub fn miniextendr(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // If not a function, delegate to ALTREP path (allow structs/enums)
    if syn::parse::<syn::ItemFn>(item.clone()).is_err() {
        return expand_altrep_struct(attr, item);
    }

    // Parse function-level attributes:
    // - #[miniextendr(unsafe(main_thread))] - forces main thread execution (unsafe context)
    // - #[miniextendr(invisible)] / #[miniextendr(visible)] - visibility control
    // - #[miniextendr(check_interrupt)] - check for Ctrl+C before execution
    // - #[miniextendr(coerce)] - enable type coercion for ALL non-R-native parameter types
    // - #[miniextendr(coerce)] on individual params - enable type coercion for specific parameters
    let mut force_main_thread = false;
    let mut force_invisible: Option<bool> = None;
    let mut check_interrupt = false;
    let mut coerce_all = false;
    if !attr.is_empty() {
        let attr_metas = syn::parse_macro_input!(attr with syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated);
        for meta in attr_metas {
            match &meta {
                // Simple identifiers: invisible, visible, check_interrupt, coerce
                syn::Meta::Path(path) => {
                    if let Some(ident) = path.get_ident() {
                        if ident == "invisible" {
                            force_invisible = Some(true);
                        } else if ident == "visible" {
                            force_invisible = Some(false);
                        } else if ident == "check_interrupt" {
                            check_interrupt = true;
                        } else if ident == "main_thread" {
                            // Legacy support - but warn/error in future?
                            force_main_thread = true;
                        } else if ident == "coerce" {
                            coerce_all = true;
                        }
                    }
                }
                // Nested: unsafe(main_thread), coerce(x, y)
                syn::Meta::List(list) => {
                    if list.path.is_ident("unsafe") {
                        let nested: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]> = list
                            .parse_args_with(syn::punctuated::Punctuated::parse_terminated)
                            .expect("expected identifier in unsafe(...)");
                        for ident in nested {
                            if ident == "main_thread" {
                                force_main_thread = true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut item = syn::parse_macro_input!(item as syn::ItemFn);

    // Collect which parameters have #[miniextendr(coerce)] attribute BEFORE stripping
    let per_param_coerce: std::collections::HashSet<String> = item
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                if has_miniextendr_coerce_attr(&pat_type.attrs) {
                    if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                        return Some(pat_ident.ident.to_string());
                    }
                }
            }
            None
        })
        .collect();

    // Transform `_` wildcard patterns to synthetic identifiers `__unused0`, `__unused1`, etc.
    // This is needed because `_` doesn't bind to a variable, but we need parameter names
    // for the C wrapper and R wrapper.
    // Also strip `#[miniextendr(coerce)]` attributes from arguments (we consume them).
    let mut unused_counter = 0usize;
    for arg in &mut item.sig.inputs {
        #[allow(clippy::collapsible_if)]
        if let syn::FnArg::Typed(pat_type) = arg {
            // Strip #[miniextendr(coerce)] attribute - we consume it, don't pass through
            pat_type.attrs.retain(|attr| {
                if attr.path().is_ident("miniextendr") {
                    if let syn::Meta::List(list) = &attr.meta {
                        if let Ok(nested) = list.parse_args::<syn::Ident>() {
                            if nested == "coerce" {
                                return false; // Remove this attribute
                            }
                        }
                    }
                }
                true // Keep other attributes
            });

            if matches!(pat_type.pat.as_ref(), syn::Pat::Wild(_)) {
                let synthetic_name = format!("__unused{}", unused_counter);
                unused_counter += 1;
                let synthetic_ident = syn::Ident::new(&synthetic_name, pat_type.pat.span());
                *pat_type.pat = syn::Pat::Ident(syn::PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: None,
                    ident: synthetic_ident,
                    subpat: None,
                });
            }
        }
    }

    // dots support here
    //TODO: move to ExtendrFunction?
    let has_dots = item.sig.variadic.is_some();
    let mut named_dots: Option<syn::Ident> = if has_dots {
        let dots = item.sig.variadic.as_ref().unwrap();
        if let Some(named_dots) = dots.pat.as_ref() {
            if let syn::Pat::Ident(named_dots_ident) = named_dots.0.as_ref() {
                Some(named_dots_ident.ident.clone())
            } else {
                // Pattern match on dots that isn't a simple ident (e.g., `(a, b): ...`)
                // This is not supported in R's ... semantics
                panic!(
                    "variadic pattern must be a simple identifier, got: {:?}",
                    named_dots.0
                );
            }
        } else {
            // unnamed dots
            None
        }
    } else {
        None
    };
    item.sig.variadic = None;
    // instead of ... replace with Dots type!
    //TODO: investigate why this needs to be &Dots.
    if has_dots {
        item.sig
            .inputs
            .push(if let Some(named_dots) = named_dots.as_ref() {
                syn::parse_quote!(#named_dots: &::miniextendr_api::dots::Dots)
            } else {
                // cannot use `_` as variable name, thus cannot use it as a placeholder for `...`
                // Check that no existing parameter is named `_dots`
                for arg in &item.sig.inputs {
                    #[allow(clippy::collapsible_if)]
                    if let syn::FnArg::Typed(pat_type) = arg {
                        if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                            if pat_ident.ident == "_dots" {
                                panic!("parameter named `_dots` conflicts with implicit dots parameter; use named dots like `my_dots: ...` instead");
                            }
                        }
                    }
                }
                syn::parse_quote!(_dots: &::miniextendr_api::dots::Dots)
            });
    }
    let original_item = item.clone();
    use quote::ToTokens;
    let item = item.into_token_stream();
    let extendr_function = syn::parse2(item).unwrap();
    let ExtendrFunction {
        attrs,
        vis,
        abi,
        ident,
        generics,
        inputs,
        output,
    } = extendr_function;
    use syn::spanned::Spanned;
    let uses_internal_c_wrapper = abi.is_none();
    let rust_arg_count = inputs.len();
    let registered_arg_count = if uses_internal_c_wrapper {
        rust_arg_count + 1
    } else {
        rust_arg_count
    };
    let num_args = syn::LitInt::new(&registered_arg_count.to_string(), inputs.span());

    let rust_ident = &ident;
    let call_method_def = quote::format_ident!("call_method_def_{rust_ident}");
    let c_ident = if uses_internal_c_wrapper {
        &quote::format_ident!("C_{rust_ident}")
    } else {
        rust_ident
    };

    // name of the C-wrapper
    let c_ident_name = syn::LitCStr::new(
        std::ffi::CString::new(c_ident.to_string())
            .expect("couldn't crate a C-string for the C wrapper name")
            .as_c_str(),
        ident.span(),
    );
    // registration of the C-wrapper
    // these are needed to transmute fn-item to fn-pointer correctly.
    let mut func_ptr_def: Vec<syn::Pat> = Vec::new();
    if uses_internal_c_wrapper {
        func_ptr_def.push(syn::parse_quote!(::miniextendr_api::ffi::SEXP));
    }
    func_ptr_def.extend(
        (0..inputs.len())
            .map(|_| syn::parse_quote!(::miniextendr_api::ffi::SEXP))
            .collect::<Vec<_>>(),
    );

    // calling the rust function with
    let rust_inputs: Vec<syn::Ident> = inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(p) = pt.pat.as_ref()
            {
                return Some(p.ident.clone());
            }
            None
        })
        .collect();
    // dbg!(&rust_inputs);

    // calling the C-wrapper with
    let call_param_ident = syn::Ident::new("__miniextendr_call", proc_macro2::Span::call_site());
    let mut c_wrapper_inputs: Vec<_> = Vec::new();
    if uses_internal_c_wrapper {
        c_wrapper_inputs.push(syn::parse_quote!(#call_param_ident: ::miniextendr_api::ffi::SEXP));
    }
    c_wrapper_inputs.extend(inputs.clone().into_pairs().map(|pair| {
        let arg = pair.value();
        match arg {
            syn::FnArg::Receiver(receiver) => {
                syn::Error::new(receiver.span(), "impl-blocks not supported yet").to_compile_error()
            }
            syn::FnArg::Typed(pt) => {
                let syn::PatType {
                    attrs: _,
                    pat,
                    colon_token: _,
                    ty: _,
                } = pt;
                match pat.as_ref() {
                    syn::Pat::Ident(pat_ident) => {
                        let mut pat_ident = pat_ident.clone();
                        pat_ident.mutability = None;
                        pat_ident.by_ref = None;
                        let ident = pat_ident;
                        syn::parse_quote!(#ident: ::miniextendr_api::ffi::SEXP)
                    }
                    syn::Pat::Wild(_) => {
                        unreachable!("wildcard patterns should have been transformed to synthetic identifiers")
                    }
                    _ => {
                        panic!("unsupported pattern in function argument: {:?}", pat)
                    }
                }
            }
        }
    }));
    // dbg!(&wrapper_inputs);
    let mut pre_call_statements: Vec<proc_macro2::TokenStream> = Vec::new();
    if check_interrupt {
        pre_call_statements.push(quote::quote! {
            unsafe { ::miniextendr_api::ffi::R_CheckUserInterrupt(); }
        });
    }
    let mut closure_statements: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut post_call_statements: Vec<proc_macro2::TokenStream> = Vec::new();
    for arg in inputs.iter() {
        let syn::FnArg::Typed(pat_type) = arg else {
            // TODO: no support for self!
            continue;
        };
        let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            continue;
        };
        let ident = pat_ident.ident.clone();

        match pat_type.ty.as_ref() {
            syn::Type::Tuple(t) if t.elems.is_empty() => {
                if pat_ident.mutability.is_some() {
                    closure_statements.push(quote::quote! { let mut #ident = (); });
                } else {
                    closure_statements.push(quote::quote! { let #ident = (); });
                }
            }
            syn::Type::Reference(r) => {
                let is_dots = matches!(
                    r.elem.as_ref(),
                    syn::Type::Path(tp)
                        if tp
                            .path
                            .segments
                            .last()
                            .map(|s| s.ident == "Dots")
                            .unwrap_or(false)
                );
                let is_slice = matches!(r.elem.as_ref(), syn::Type::Slice(_));
                // Check for &str - strings need TryFromSexp, not DATAPTR_RO
                let is_str = matches!(
                    r.elem.as_ref(),
                    syn::Type::Path(tp) if tp.path.is_ident("str")
                );
                if is_dots {
                    let storage_ident = quote::format_ident!("{}_storage", ident);
                    closure_statements.push(quote::quote! {
                        let #storage_ident = ::miniextendr_api::dots::Dots { inner: #ident };
                        let #ident = &#storage_ident;
                    });
                } else if is_slice || is_str {
                    // Slice references and &str use TryFromSexp
                    // Strings must use STRING_ELT + R_CHAR, not DATAPTR_RO
                    closure_statements.push(quote::quote! {
                        let #ident = ::miniextendr_api::TryFromSexp::try_from_sexp(#ident).unwrap();
                    });
                } else if pat_ident.mutability.is_some() {
                    closure_statements.push(quote::quote! {
                        let mut #ident = unsafe { *::miniextendr_api::ffi::DATAPTR_unchecked(#ident).cast() };
                    });
                } else {
                    closure_statements.push(quote::quote! {
                        let #ident = unsafe { *::miniextendr_api::ffi::DATAPTR_RO_unchecked(#ident).cast() };
                    });
                }
            }
            _ => {
                // Check if coercion is enabled for this parameter:
                // - coerce_all: #[miniextendr(coerce)] on function applies to all params
                // - per_param_coerce: #[miniextendr(coerce)] on individual param
                let param_name = ident.to_string();
                let should_coerce = coerce_all || per_param_coerce.contains(&param_name);
                let coercion_mapping = if should_coerce {
                    get_coercion_mapping(pat_type.ty.as_ref())
                } else {
                    None
                };

                match coercion_mapping {
                    Some(CoercionMapping::Scalar { r_native, target }) => {
                        // Scalar coercion: extract R native, coerce to target
                        let mutability = if pat_ident.mutability.is_some() {
                            quote::quote!(mut)
                        } else {
                            quote::quote!()
                        };
                        closure_statements.push(quote::quote! {
                            let #mutability #ident: #target = {
                                let __r_val: #r_native = ::miniextendr_api::TryFromSexp::try_from_sexp(#ident).unwrap();
                                ::miniextendr_api::TryCoerce::<#target>::try_coerce(__r_val)
                                    .expect(concat!("coercion to ", stringify!(#target), " failed"))
                            };
                        });
                    }
                    Some(CoercionMapping::Vec { r_native_elem, target_elem }) => {
                        // Vec coercion: extract R native slice, coerce element-wise
                        let mutability = if pat_ident.mutability.is_some() {
                            quote::quote!(mut)
                        } else {
                            quote::quote!()
                        };
                        closure_statements.push(quote::quote! {
                            let #mutability #ident: Vec<#target_elem> = {
                                let __r_slice: &[#r_native_elem] = ::miniextendr_api::TryFromSexp::try_from_sexp(#ident).unwrap();
                                __r_slice.iter().copied()
                                    .map(::miniextendr_api::TryCoerce::<#target_elem>::try_coerce)
                                    .collect::<Result<Vec<_>, _>>()
                                    .expect(concat!("coercion to Vec<", stringify!(#target_elem), "> failed"))
                            };
                        });
                    }
                    None => {
                        // No coercion - use standard TryFromSexp
                        if pat_ident.mutability.is_some() {
                            closure_statements.push(quote::quote! {
                                let mut #ident = ::miniextendr_api::TryFromSexp::try_from_sexp(#ident).unwrap();
                            });
                        } else {
                            closure_statements.push(quote::quote! {
                                let #ident = ::miniextendr_api::TryFromSexp::try_from_sexp(#ident).unwrap();
                            });
                        }
                    }
                }
            }
        }
    }

    // Automatic invisibility detection based on return type.
    // Can be overridden with #[miniextendr(invisible)] or #[miniextendr(visible)].
    let is_invisible_return_type: bool;
    let rust_result_ident =
        syn::Ident::new("__miniextendr_rust_result", proc_macro2::Span::mixed_site());
    let option_none_error_msg = quote::quote! {
        concat!(
            "miniextendr function `",
            stringify!(#rust_ident),
            "` returned None"
        )
    };

    // Generate return expression (converts Rust result to SEXP)
    // Also track whether return type involves SEXP (can't use worker strategy for those)
    let mut returns_sexp = false;
    let return_expression = match &output {
        // no arrow
        syn::ReturnType::Default => {
            is_invisible_return_type = true;
            quote::quote! { unsafe { ::miniextendr_api::ffi::R_NilValue } }
        }

        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            // -> ()
            syn::Type::Tuple(t) if t.elems.is_empty() => {
                is_invisible_return_type = true;
                quote::quote! { unsafe { ::miniextendr_api::ffi::R_NilValue } }
            }
            syn::Type::Path(_p) if is_sexp_type(ty.as_ref()) => {
                is_invisible_return_type = false;
                returns_sexp = true;
                quote::quote! { #rust_result_ident }
            }
            // -> ExternalPtr<T> (contains SEXP, must run on main thread)
            syn::Type::Path(_p) if is_external_ptr_type(ty.as_ref()) => {
                is_invisible_return_type = false;
                returns_sexp = true; // ExternalPtr contains SEXP, forces main_thread
                quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
            }

            // -> Option<...> cases
            syn::Type::Path(p)
                if p.path.segments.last().map(|s| &s.ident)
                    == Some(&syn::Ident::new("Option", p.path.span())) =>
            {
                let seg = p.path.segments.last().unwrap();
                let inner_ty = first_type_argument(seg);
                let is_unit_inner = inner_ty.is_some_and(is_unit_type);
                let is_sexp_inner = inner_ty.is_some_and(is_sexp_type);
                let is_extptr_inner = inner_ty.is_some_and(is_external_ptr_type);

                if is_unit_inner {
                    is_invisible_return_type = true;
                    post_call_statements.push(quote::quote! {
                        if #rust_result_ident.is_none() {
                            panic!(#option_none_error_msg);
                        }
                    });
                    quote::quote! { unsafe { ::miniextendr_api::ffi::R_NilValue } }
                } else {
                    is_invisible_return_type = false;
                    if is_sexp_inner || is_extptr_inner {
                        returns_sexp = true;
                    }
                    post_call_statements.push(quote::quote! {
                        let #rust_result_ident = match #rust_result_ident {
                            Some(v) => v,
                            None => panic!(#option_none_error_msg),
                        };
                    });
                    if is_sexp_inner {
                        quote::quote! { #rust_result_ident }
                    } else {
                        quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
                    }
                }
            }

            // -> Result<...> cases
            syn::Type::Path(p)
                if p.path.segments.last().map(|s| &s.ident)
                    == Some(&syn::Ident::new("Result", p.path.span())) =>
            {
                let seg = p.path.segments.last().unwrap();
                let ok_ty = first_type_argument(seg);
                let ok_is_unit = ok_ty.is_some_and(is_unit_type);
                let ok_is_sexp = ok_ty.is_some_and(is_sexp_type);
                let ok_is_extptr = ok_ty.is_some_and(is_external_ptr_type);

                if ok_is_unit {
                    is_invisible_return_type = true;
                    post_call_statements.push(quote::quote! {
                        if let Err(e) = #rust_result_ident {
                            panic!("{:?}", e);
                        }
                    });
                    quote::quote! { unsafe { ::miniextendr_api::ffi::R_NilValue } }
                } else {
                    is_invisible_return_type = false;
                    if ok_is_sexp || ok_is_extptr {
                        returns_sexp = true;
                    }
                    post_call_statements.push(quote::quote! {
                        let #rust_result_ident = match #rust_result_ident {
                            Ok(v) => v,
                            Err(e) => panic!("{:?}", e),
                        };
                    });
                    if ok_is_sexp {
                        quote::quote! { #rust_result_ident }
                    } else {
                        quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
                    }
                }
            }

            // all other T
            _ => {
                is_invisible_return_type = false;
                quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
            }
        },
    };

    // Apply explicit visibility override from #[miniextendr(invisible)] or #[miniextendr(visible)]
    let is_invisible_return_type = force_invisible.unwrap_or(is_invisible_return_type);

    // Use worker strategy by default for functions that don't return SEXP.
    // Worker thread provides proper panic catching with destructor cleanup.
    // Functions returning SEXP or taking Dots must stay on main thread (SEXP/Dots aren't Send).
    // Use #[miniextendr(main_thread)] to force main thread for functions that call R APIs internally.
    // check_interrupt also requires main thread since R_CheckUserInterrupt must be called there.
    let use_main_thread = returns_sexp || has_dots || force_main_thread || check_interrupt;
    let c_wrapper = if abi.is_some() {
        proc_macro2::TokenStream::new()
    } else if use_main_thread {
        // SEXP-returning or Dots-taking functions: use with_r_unwind_protect on main thread
        let c_wrapper_doc = format!("C wrapper for [`{}`] (main thread).", rust_ident);
        quote::quote! {
            #[doc = #c_wrapper_doc]
            #[unsafe(no_mangle)]
            #vis extern "C-unwind" fn #c_ident #generics(#(#c_wrapper_inputs),*) -> ::miniextendr_api::ffi::SEXP {
                #(#pre_call_statements)*

                ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                    || {
                        #(#closure_statements)*
                        let #rust_result_ident = #rust_ident(#(#rust_inputs),*);
                        #(#post_call_statements)*
                        #return_expression
                    },
                    Some(#call_param_ident),
                )
            }
        }
    } else {
        // Pure Rust functions: use worker thread strategy
        // 1. Argument conversion on main thread
        // 2. Function execution + Option/Result handling on worker thread
        // 3. SEXP conversion on main thread (protected by with_r_unwind_protect)
        //
        // The entire body is wrapped in catch_unwind to catch panics from:
        // - TryFromSexp::try_from_sexp().unwrap() (argument conversion)
        // - IntoR::into_sexp() (result conversion) - also wrapped in with_r_unwind_protect
        //   to catch R errors (longjmp) from SEXP creation (e.g., allocation failure)
        let c_wrapper_doc = format!("C wrapper for [`{}`] (worker thread).", rust_ident);
        quote::quote! {
            #[doc = #c_wrapper_doc]
            #[unsafe(no_mangle)]
            #vis extern "C-unwind" fn #c_ident #generics(#(#c_wrapper_inputs),*) -> ::miniextendr_api::ffi::SEXP {
                let __miniextendr_panic_result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(move || {
                    #(#pre_call_statements)*
                    #(#closure_statements)*

                    let #rust_result_ident = ::miniextendr_api::worker::run_on_worker(move || {
                        let #rust_result_ident = #rust_ident(#(#rust_inputs),*);
                        #(#post_call_statements)*
                        #rust_result_ident
                    });

                    // Wrap SEXP conversion in with_r_unwind_protect to catch R errors
                    // (e.g., allocation failure in Rf_ScalarString)
                    ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                        move || #return_expression,
                        None,
                    )
                }));
                match __miniextendr_panic_result {
                    Ok(sexp) => sexp,
                    Err(payload) => ::miniextendr_api::worker::panic_message_to_r_error(
                        ::miniextendr_api::worker::panic_payload_to_string(&payload)
                    ),
                }
            }
        }
    };

    // check the validity of the provided C-function!
    if abi.is_some() {
        // check that #[no_mangle] or #[unsafe(no_mangle)] is present!
        let has_no_mangle = attrs.iter().any(|attr| {
            attr.path().is_ident("no_mangle")
                || attr
                    .parse_nested_meta(|meta| {
                        if meta.path.is_ident("no_mangle") {
                            Err(meta.error("found #[no_mangle]"))
                        } else {
                            Ok(())
                        }
                    })
                    .is_err()
        });

        if !has_no_mangle {
            return syn::Error::new(
                attrs
                    .first()
                    .map(|attr| attr.span())
                    .unwrap_or_else(|| abi.span()),
                "missing #[no_mangle] (edition 2021) or #[unsafe(no_mangle)] (edition 2024).",
            )
            .into_compile_error()
            .into();
        }

        // Validate return type is SEXP for extern "C-unwind" functions
        match output {
            non_return_type @ syn::ReturnType::Default => {
                return syn::Error::new(non_return_type.span(), "output must be SEXP")
                    .into_compile_error()
                    .into();
            }
            syn::ReturnType::Type(_rarrow, output_type) => match output_type.as_ref() {
                syn::Type::Path(type_path) => {
                    if let Some(path_to_sexp) = type_path.path.segments.last().map(|x| &x.ident)
                        && path_to_sexp != "SEXP"
                    {
                        return syn::Error::new(path_to_sexp.span(), "output must be SEXP")
                            .into_compile_error()
                            .into();
                    }
                }
                _ => {
                    return syn::Error::new(output_type.span(), "output must be SEXP")
                        .into_compile_error()
                        .into();
                }
            },
        }
    }

    // region: R wrappers generation in `fn`
    // normalize `named_dots` for R (no leading underscore)
    if has_dots && let Some(named) = &mut named_dots {
        let mut arg_name = named.to_string();
        if arg_name.starts_with("__") {
            arg_name.insert_str(0, "private");
        } else if arg_name.starts_with('_') {
            arg_name.insert_str(0, "unused");
        }
        let arg_ident = syn::Ident::new(&arg_name, named.span());
        *named = arg_ident;
    }

    // Build both the .Call argument list and the formal parameter list in one pass
    let last_idx = inputs.len().saturating_sub(1);
    let mut r_call_args_strs: Vec<String> = Vec::new();
    if uses_internal_c_wrapper {
        r_call_args_strs.push(".call = match.call()".to_string());
    }
    let mut r_formals: Vec<proc_macro2::TokenStream> = Vec::new();
    for (idx, x) in inputs.iter().enumerate() {
        let syn::FnArg::Typed(pat_type) = x else {
            unreachable!()
        };
        let syn::PatType {
            attrs: _,
            pat,
            colon_token: _,
            ty,
        } = pat_type;

        // derive R argument name, applying leading-underscore rename
        let arg_ident = match pat.as_ref() {
            syn::Pat::Ident(pat_ident) => {
                let mut arg_name = pat_ident.ident.to_string();
                if arg_name.starts_with("__") {
                    arg_name.insert_str(0, "private");
                } else if arg_name.starts_with('_') {
                    arg_name.insert_str(0, "unused");
                }
                syn::Ident::new(&arg_name, pat_ident.ident.span())
            }
            _ => unreachable!(),
        };

        // call-site argument
        if has_dots && idx == last_idx {
            if let Some(named_dots) = &named_dots {
                r_call_args_strs.push(format!("list({})", named_dots));
            } else {
                r_call_args_strs.push("list(...)".to_string());
            }
        } else {
            r_call_args_strs.push(arg_ident.to_string());
        }

        // formal parameter (with defaults for unit types)
        if has_dots && idx == last_idx {
            if let Some(named_dots) = &named_dots {
                let named = syn::Ident::new(&named_dots.to_string(), named_dots.span());
                r_formals.push(syn::parse_quote!(#named = ...));
            } else {
                r_formals.push(syn::parse_quote!(...));
            }
        } else {
            match ty.as_ref() {
                syn::Type::Tuple(t) if t.elems.is_empty() => {
                    r_formals.push(syn::parse_quote!(#arg_ident = NULL));
                }
                _ => {
                    r_formals.push(arg_ident.into_token_stream());
                }
            }
        }
    }

    // region: R wrappers generation in `fn`
    // Build the R body string consistently
    let c_ident_str = c_ident.to_string();
    let call_args_joined = r_call_args_strs.join(", ");
    let call_expr = if r_call_args_strs.is_empty() {
        format!(".Call({})", c_ident_str)
    } else {
        format!(".Call({}, {})", c_ident_str, call_args_joined)
    };
    let r_wrapper_return_str = if !is_invisible_return_type {
        call_expr
    } else {
        format!("invisible({})", call_expr)
    };
    let r_wrapper_ident = if abi.is_some() {
        &quote::format_ident!("unsafe_{rust_ident}")
    } else {
        rust_ident
    };
    // Stable, consistent R formatting style: brace on same line, body indented, closing brace on its own line
    let formals_joined = r_formals
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    // Add #' @export roxygen comment if function is `pub` (not pub(crate), pub(super), etc.)
    let export_comment = if is_pub_visibility(&vis) {
        "#' @export\n"
    } else {
        ""
    };
    let r_wrapper_string = format!(
        "{}{} <- function({}) {{\n    {}\n}}",
        export_comment, r_wrapper_ident, formals_joined, r_wrapper_return_str
    );
    // Use a raw string literal for better readability in macro expansion
    let r_wrapper_str: proc_macro2::TokenStream = {
        use std::str::FromStr;
        // Indent each line by 4 spaces for nicer formatting
        let indented = r_wrapper_string.replace('\n', "\n    ");
        let raw = format!("r#\"\n    {}\n\"#", indented);
        proc_macro2::TokenStream::from_str(&raw).expect("valid raw string literal")
    };

    let rust_ident_upper = rust_ident.to_string().to_uppercase();
    let r_wrapper_generator = quote::format_ident!("R_WRAPPER_{rust_ident_upper}");

    // endregion

    let abi = abi.unwrap_or(syn::parse_quote!(extern "C-unwind"));

    // Generate doc strings with links
    let r_wrapper_doc = format!(
        "R wrapper code for [`{}`], calls [`{}`].",
        rust_ident, c_ident
    );
    let call_method_def_doc = format!(
        "R call method definition for [`{}`] (C wrapper: [`{}`]).",
        rust_ident, c_ident
    );

    let expanded: proc_macro::TokenStream = quote::quote! {
        // rust function!
        #original_item

        // C wrapper
        #c_wrapper

        // R wrapper
        #[doc = #r_wrapper_doc]
        const #r_wrapper_generator: &str = #r_wrapper_str;

        // registration of C wrapper in R
        #[doc = #call_method_def_doc]
        #[inline(always)]
        #[allow(non_snake_case)]
        const fn #call_method_def() -> ::miniextendr_api::ffi::R_CallMethodDef {
            unsafe {
                ::miniextendr_api::ffi::R_CallMethodDef {
                    name: #c_ident_name.as_ptr(),
                    fun: Some(std::mem::transmute::<
                        unsafe #abi fn(#(#func_ptr_def),*) -> ::miniextendr_api::ffi::SEXP,
                        unsafe #abi fn(...) -> ::miniextendr_api::ffi::SEXP
                    >(#c_ident)),
                    numArgs: #num_args,
                }
            }
        }
    }
    .into();

    expanded
}

struct ExtendrModuleFunction {
    pub _abi: Option<syn::Abi>,
    _fn_token: syn::Token![fn],
    pub ident: syn::Ident,
}

impl syn::parse::Parse for ExtendrModuleFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _abi = if input.peek(syn::Token![extern]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            _abi,
            _fn_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

struct ExtendrModuleStruct {
    _struct_token: syn::Token![struct],
    #[allow(dead_code)]
    pub ident: syn::Ident,
}

impl syn::parse::Parse for ExtendrModuleStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _struct_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

struct ExtendrModuleName {
    _mod_token: syn::Token![mod],
    pub ident: syn::Ident,
}

impl syn::parse::Parse for ExtendrModuleName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _mod_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

struct ExtendrModuleUse {
    _use_token: syn::Token![use],
    pub use_name: syn::UseName,
}

impl syn::parse::Parse for ExtendrModuleUse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::spanned::Spanned;
        let _use_token = input.parse()?;
        let use_name: syn::UseTree = input.parse()?;
        // dbg!(&use_name);
        let use_name = match use_name {
            syn::UseTree::Name(use_name) => use_name,
            // TODO: provide boilerplate error message here.
            syn::UseTree::Rename(use_rename) => {
                return Err(syn::Error::new(
                    use_rename.span(),
                    "it is not possible to rename wrappers in `miniextendr_module`",
                ));
            }
            syn::UseTree::Path(_) | syn::UseTree::Glob(_) | syn::UseTree::Group(_) => {
                return Err(syn::Error::new(use_name.span(), "syntax not supported"));
            }
        };
        Ok(Self {
            _use_token,
            use_name,
        })
    }
}

struct ExtendrModule {
    pub extendr_module: ExtendrModuleName,
    pub extendr_use: Vec<ExtendrModuleUse>,
    pub extendr_fn: Vec<ExtendrModuleFunction>,
    pub extendr_struct: Vec<ExtendrModuleStruct>,
    // TODO: add extendr_impl: Vec<ExtendrImpl>
}

enum ExtendrItem {
    Module(ExtendrModuleName),
    Use(ExtendrModuleUse),
    Struct(ExtendrModuleStruct),
    Func(ExtendrModuleFunction),
}

impl syn::parse::Parse for ExtendrItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let look_ahead = input.lookahead1();
        if look_ahead.peek(syn::Token![mod]) {
            Ok(Self::Module(input.parse()?))
        } else if look_ahead.peek(syn::Token![use]) {
            Ok(Self::Use(input.parse()?))
        } else if look_ahead.peek(syn::Token![struct]) {
            Ok(Self::Struct(input.parse()?))
        } else if look_ahead.peek(syn::Token![fn]) || look_ahead.peek(syn::Token![extern]) {
            Ok(Self::Func(input.parse()?))
        } else {
            Err(look_ahead.error())
        }
    }
}

impl syn::parse::Parse for ExtendrModule {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let items: syn::punctuated::Punctuated<ExtendrItem, syn::Token![;]> =
            syn::punctuated::Punctuated::parse_terminated_with(input, ExtendrItem::parse)?;

        let mut name = None;
        let mut uses = Vec::new();
        let mut funs = Vec::new();
        let mut structs = Vec::new();

        for it in items {
            match it {
                ExtendrItem::Module(m) => {
                    if name.is_some() {
                        return Err(syn::Error::new(m._mod_token.span, "duplicate `mod <name>`"));
                    }
                    name = Some(m);
                }
                ExtendrItem::Use(u) => uses.push(u),
                ExtendrItem::Struct(s) => structs.push(s),
                ExtendrItem::Func(f) => funs.push(f),
            }
        }

        let extendr_module =
            name.ok_or_else(|| syn::Error::new(input.span(), "missing `mod <name>`"))?;

        Ok(Self {
            extendr_module,
            extendr_use: uses,
            extendr_fn: funs,
            extendr_struct: structs,
        })
    }
}

// TODO: Currently, miniextendr_module does not distinguish between
// `extern "C-unwind" fn` and `fn` items.. they are treated alike.
#[proc_macro]
pub fn miniextendr_module(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let miniextendr_module = syn::parse_macro_input!(item as ExtendrModule);

    let module = miniextendr_module.extendr_module.ident;
    let module_entrypoint_ident = quote::format_ident!("R_init_{module}_miniextendr");
    let call_entries: Vec<syn::Expr> = miniextendr_module
        .extendr_fn
        .iter()
        .map(|miniextendr_fn| {
            //TODO: put this in ExtendrFunction impl
            let rust_ident = &miniextendr_fn.ident;
            let call_method_def = quote::format_ident!("call_method_def_{rust_ident}");
            syn::parse_quote!(#call_method_def())
        })
        .collect();
    let call_entries_len = call_entries.len();

    // Generate ALTREP registrations for struct items (if they implement RegisterAltrep)
    let altrep_regs: Vec<syn::Expr> = miniextendr_module
        .extendr_struct
        .iter()
        .map(|s| {
            let ty = &s.ident;
            syn::parse_quote!(<#ty as ::miniextendr_api::altrep_registration::RegisterAltrep>::get_or_init_class())
        })
        .collect();

    // call the R_init from all the submodules (given by `use`)
    let use_other_modules = miniextendr_module
        .extendr_use
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let use_module_init = quote::format_ident!("R_init_{use_module_ident}_miniextendr");
            syn::parse_quote!(#use_module_ident::#use_module_init(dll))
        })
        .collect::<Vec<syn::Expr>>();

    // region: r wrapper generation in `mod`

    let r_wrapper_generators: Vec<syn::Expr> = miniextendr_module
        .extendr_fn
        .iter()
        .map(|x| {
            //TODO: put this in ExtendrFunction impl
            let rust_ident = &x.ident;
            let rust_ident_upper = rust_ident.to_string().to_uppercase();
            let r_wrapper_generator = quote::format_ident!("R_WRAPPER_{rust_ident_upper}");
            syn::parse_quote!(#r_wrapper_generator)
        })
        .collect();
    let r_wrappers_use_other_modules = miniextendr_module
        .extendr_use
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let use_module_ident_upper = use_module_ident.to_string().to_uppercase();
            let r_wrappers_use_module =
                quote::format_ident!("R_WRAPPERS_PARTS_{use_module_ident_upper}");
            syn::parse_quote!(#use_module_ident::#r_wrappers_use_module)
        })
        .collect::<Vec<syn::Expr>>();

    let module_upper = module.to_string().to_uppercase();
    let r_wrappers_parts_ident = quote::format_ident!("R_WRAPPERS_PARTS_{module_upper}");
    let r_wrappers_deps_ident = quote::format_ident!("R_WRAPPERS_DEPS_{module_upper}");

    // Generate doc string listing all registered functions
    let fn_links: Vec<String> = miniextendr_module
        .extendr_fn
        .iter()
        .map(|f| format!("[`{}`]", f.ident))
        .collect();
    let struct_links: Vec<String> = miniextendr_module
        .extendr_struct
        .iter()
        .map(|s| format!("[`{}`]", s.ident))
        .collect();
    let module_doc = if fn_links.is_empty() && struct_links.is_empty() {
        format!("R entrypoint for module `{}`.", module)
    } else {
        let mut doc = format!(
            "R entrypoint for module `{}`.\n\n# Registered items\n",
            module
        );
        if !fn_links.is_empty() {
            doc.push_str(&format!("- Functions: {}\n", fn_links.join(", ")));
        }
        if !struct_links.is_empty() {
            doc.push_str(&format!("- ALTREP types: {}\n", struct_links.join(", ")));
        }
        doc
    };

    // endregion
    quote::quote! {

        #[doc(hidden)]
        pub const #r_wrappers_parts_ident: &[&str] = &[#(#r_wrapper_generators),*];
        #[doc(hidden)]
        pub const #r_wrappers_deps_ident: &[&[&str]] = &[#(#r_wrappers_use_other_modules),*];

        #[doc = #module_doc]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub(crate) extern "C-unwind" fn #module_entrypoint_ident(dll: *mut ::miniextendr_api::ffi::DllInfo) {
            static CALL_ENTRIES: [::miniextendr_api::ffi::R_CallMethodDef; {#call_entries_len + 1}] = [
                #(#call_entries,)*
                ::miniextendr_api::ffi::R_CallMethodDef {
                    name: std::ptr::null(),
                    fun: None,
                    numArgs: 0,
                }
            ];

            #(#use_other_modules;)*

            // Register any ALTREP classes declared as struct items in this module
            #(#altrep_regs;)*

            unsafe {
                ::miniextendr_api::ffi::R_registerRoutines_unchecked(dll, std::ptr::null(), CALL_ENTRIES.as_ptr(), std::ptr::null(), std::ptr::null());
                // these are already present in entrypoint.c!
                // R_useDynamicSymbols(dll, Rboolean::FALSE);
                // R_forceSymbols(dll, Rboolean::TRUE);
            }
        }
    }
    .into()
}

/// Internal: expand ALTREP struct registration for #[miniextendr] when used on a type.
///
/// Requires a 1-field wrapper struct like `struct MyInts(Vec<i32>);`
/// The inner type must implement `AltrepBase` and the appropriate data traits.
fn expand_altrep_struct(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    use syn::spanned::Spanned;
    let input: syn::ItemStruct = match syn::parse(item.clone()) {
        Ok(it) => it,
        Err(e) => return e.into_compile_error().into(),
    };

    let ident = input.ident.clone();
    let generics = input.generics.clone();

    // Extract the inner type - must be a 1-field struct
    let data_ty: syn::Type = match &input.fields {
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            fields.unnamed.first().unwrap().ty.clone()
        }
        syn::Fields::Named(fields) if fields.named.len() == 1 => {
            fields.named.first().unwrap().ty.clone()
        }
        _ => {
            return syn::Error::new(
                input.span(),
                "#[miniextendr] ALTREP requires a 1-field wrapper struct, e.g., `struct MyInts(Vec<i32>);`",
            )
            .into_compile_error()
            .into();
        }
    };

    // Parse attr list: class = "...", pkg = "..."
    // base is optional - inferred from AltrepBase if not provided
    use syn::parse::Parser;
    let parser =
        syn::punctuated::Punctuated::<syn::MetaNameValue, syn::Token![,]>::parse_terminated;
    let args = match parser.parse(attr) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };
    let mut class_name = None::<String>;
    let mut pkg_name = None::<String>;
    let mut base_name = None::<String>;
    for nv in args {
        let key = nv
            .path
            .get_ident()
            .map(|i| i.to_string())
            .unwrap_or_default();
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &nv.value
        {
            match key.as_str() {
                "class" => class_name = Some(s.value()),
                "pkg" => pkg_name = Some(s.value()),
                "base" => base_name = Some(s.value()),
                _ => {}
            }
        }
    }

    let class_name = class_name.expect("#[miniextendr] missing class = \"...\"");
    let pkg_name = pkg_name.expect("#[miniextendr] missing pkg = \"...\"");
    // base is now OPTIONAL - inferred from AltrepBase if not provided

    // Validate base if provided, otherwise use AltrepBase inference
    let base_variant: syn::Expr = if let Some(ref base_name) = base_name {
        match base_name.as_str() {
            "Int" => syn::parse_quote!(::miniextendr_api::altrep::RBase::Int),
            "Real" => syn::parse_quote!(::miniextendr_api::altrep::RBase::Real),
            "Logical" => syn::parse_quote!(::miniextendr_api::altrep::RBase::Logical),
            "Raw" => syn::parse_quote!(::miniextendr_api::altrep::RBase::Raw),
            "String" => syn::parse_quote!(::miniextendr_api::altrep::RBase::String),
            "List" => syn::parse_quote!(::miniextendr_api::altrep::RBase::List),
            "Complex" => syn::parse_quote!(::miniextendr_api::altrep::RBase::Complex),
            _ => {
                return syn::Error::new_spanned(
                    syn::LitStr::new(base_name, ident.span()),
                    "base must be one of Int|Real|Logical|Raw|String|List|Complex",
                )
                .into_compile_error()
                .into();
            }
        }
    } else {
        // Infer from InferBase trait (auto-implemented via impl_inferbase_* macros)
        syn::parse_quote!(<#data_ty as ::miniextendr_api::altrep_data::InferBase>::BASE)
    };

    // The trampoline type is always the inner data type
    let tramp_ty = data_ty.clone();

    // Generate family setters and make_class based on the base type
    // If base is explicitly provided, generate type-specific code at macro time.
    // If base is not provided, use AltrepInstaller trait for compile-time dispatch.
    let (family_setters, make_class): (proc_macro2::TokenStream, proc_macro2::TokenStream) =
        if let Some(ref base_name) = base_name {
            // Explicit base: generate type-specific code at macro time
            let setters = match base_name.as_str() {
                "Int" => quote::quote! {
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_ELT, R_set_altinteger_Elt_method, bridge::t_int_elt::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_GET_REGION, R_set_altinteger_Get_region_method, bridge::t_int_get_region::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_IS_SORTED, R_set_altinteger_Is_sorted_method, bridge::t_int_is_sorted::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_NO_NA, R_set_altinteger_No_NA_method, bridge::t_int_no_na::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_SUM, R_set_altinteger_Sum_method, bridge::t_int_sum::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_MIN, R_set_altinteger_Min_method, bridge::t_int_min::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_MAX, R_set_altinteger_Max_method, bridge::t_int_max::<#tramp_ty>);
                },
                "Real" => quote::quote! {
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_ELT, R_set_altreal_Elt_method, bridge::t_real_elt::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_GET_REGION, R_set_altreal_Get_region_method, bridge::t_real_get_region::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_IS_SORTED, R_set_altreal_Is_sorted_method, bridge::t_real_is_sorted::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_NO_NA, R_set_altreal_No_NA_method, bridge::t_real_no_na::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_SUM, R_set_altreal_Sum_method, bridge::t_real_sum::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_MIN, R_set_altreal_Min_method, bridge::t_real_min::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_MAX, R_set_altreal_Max_method, bridge::t_real_max::<#tramp_ty>);
                },
                "Logical" => quote::quote! {
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltLogical>::HAS_ELT, R_set_altlogical_Elt_method, bridge::t_lgl_elt::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltLogical>::HAS_GET_REGION, R_set_altlogical_Get_region_method, bridge::t_lgl_get_region::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltLogical>::HAS_IS_SORTED, R_set_altlogical_Is_sorted_method, bridge::t_lgl_is_sorted::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltLogical>::HAS_NO_NA, R_set_altlogical_No_NA_method, bridge::t_lgl_no_na::<#tramp_ty>);
                },
                "Raw" => quote::quote! {
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltRaw>::HAS_ELT, R_set_altraw_Elt_method, bridge::t_raw_elt::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltRaw>::HAS_GET_REGION, R_set_altraw_Get_region_method, bridge::t_raw_get_region::<#tramp_ty>);
                },
                "String" => quote::quote! {
                    unsafe { R_set_altstring_Elt_method(cls, Some(bridge::t_str_elt::<#tramp_ty>)); }
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltString>::HAS_IS_SORTED, R_set_altstring_Is_sorted_method, bridge::t_str_is_sorted::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltString>::HAS_NO_NA, R_set_altstring_No_NA_method, bridge::t_str_no_na::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltString>::HAS_SET_ELT, R_set_altstring_Set_elt_method, bridge::t_str_set_elt::<#tramp_ty>);
                },
                "List" => quote::quote! {
                    unsafe { R_set_altlist_Elt_method(cls, Some(bridge::t_list_elt::<#tramp_ty>)); }
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltList>::HAS_SET_ELT, R_set_altlist_Set_elt_method, bridge::t_list_set_elt::<#tramp_ty>);
                },
                "Complex" => quote::quote! {
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltComplex>::HAS_ELT, R_set_altcomplex_Elt_method, bridge::t_cplx_elt::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltComplex>::HAS_GET_REGION, R_set_altcomplex_Get_region_method, bridge::t_cplx_get_region::<#tramp_ty>);
                },
                _ => quote::quote! {},
            };
            let make = match base_name.as_str() {
                "Int" => quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altinteger_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::PKG_NAME.as_ptr(),
                    core::ptr::null_mut(),
                ) },
                "Real" => quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altreal_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::PKG_NAME.as_ptr(),
                    core::ptr::null_mut(),
                ) },
                "Logical" => quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altlogical_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::PKG_NAME.as_ptr(),
                    core::ptr::null_mut(),
                ) },
                "Raw" => quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altraw_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::PKG_NAME.as_ptr(),
                    core::ptr::null_mut(),
                ) },
                "String" => quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altstring_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::PKG_NAME.as_ptr(),
                    core::ptr::null_mut(),
                ) },
                "List" => quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altlist_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::PKG_NAME.as_ptr(),
                    core::ptr::null_mut(),
                ) },
                "Complex" => quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altcomplex_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::PKG_NAME.as_ptr(),
                    core::ptr::null_mut(),
                ) },
                _ => quote::quote! { unreachable!() },
            };
            (setters, make)
        } else {
            // No explicit base: use InferBase trait for compile-time dispatch
            // This is auto-implemented via impl_inferbase_* macros alongside impl_alt*_from_data!
            let setters = quote::quote! {
                <#tramp_ty as ::miniextendr_api::altrep_data::InferBase>::install_methods(cls);
            };
            let make = quote::quote! {
                <#tramp_ty as ::miniextendr_api::altrep_data::InferBase>::make_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::PKG_NAME.as_ptr(),
                )
            };
            (setters, make)
        };

    // Registration: per-type; create class handle then install methods via MethodRegistrar

    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // Use LitCStr for proper C string literal generation (c"...")
    let class_cstr = syn::LitCStr::new(
        &std::ffi::CString::new(class_name.as_str()).unwrap(),
        ident.span(),
    );
    let pkg_cstr = syn::LitCStr::new(
        &std::ffi::CString::new(pkg_name.as_str()).unwrap(),
        ident.span(),
    );

    // No trait forwarding: rely on trampoline type's trait impls.
    // The ALTREP trait implementations for the data type must already exist.
    // For standard types like Vec<i32>, Vec<f64>, these are provided by miniextendr_api.
    // For custom types, users must implement the data traits themselves.

    // Generate helper methods and TryFromSexp wrappers
    let ref_ident = quote::format_ident!("{}Ref", ident);
    let mut_ident = quote::format_ident!("{}Mut", ident);
    let data_helper_impl: proc_macro2::TokenStream = {
        let ref_doc = format!(
            "Immutable reference wrapper for [`{}`] ALTREP data. Implements `TryFromSexp` and `Deref<Target = {}>`.",
            ident,
            quote::quote!(#data_ty)
        );
        let mut_doc = format!(
            "Mutable reference wrapper for [`{}`] ALTREP data. Implements `TryFromSexp`, `Deref`, and `DerefMut`.",
            ident
        );
        quote::quote! {
            impl #ident {
                /// Create an ALTREP SEXP from the given data.
                ///
                /// # Safety
                ///
                /// Must be called from the R main thread.
                pub unsafe fn into_altrep(data: #data_ty) -> ::miniextendr_api::ffi::SEXP {
                    use ::miniextendr_api::altrep_registration::RegisterAltrep;
                    use ::miniextendr_api::externalptr::ExternalPtr;
                    use ::miniextendr_api::ffi::altrep::R_new_altrep;
                    use ::miniextendr_api::ffi::R_NilValue;

                    let ext_ptr = ExternalPtr::new(data);
                    let cls = Self::get_or_init_class();
                    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
                }
            }

            #[doc = #ref_doc]
            pub struct #ref_ident(::miniextendr_api::externalptr::ExternalPtr<#data_ty>);

            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            impl ::miniextendr_api::TryFromSexp for #ref_ident {
                type Error = ::miniextendr_api::SexpTypeError;

                fn try_from_sexp(sexp: ::miniextendr_api::ffi::SEXP) -> Result<Self, Self::Error> {
                    use ::miniextendr_api::ffi::SEXPTYPE;

                    // Check if it's an ALTREP object (ALTREP returns c_int, not bool)
                    if unsafe { ::miniextendr_api::ffi::ALTREP(sexp) } == 0 {
                        return Err(::miniextendr_api::SexpTypeError {
                            expected: SEXPTYPE::INTSXP, // placeholder - ALTREP check failed
                            actual: unsafe { ::miniextendr_api::ffi::TYPEOF(sexp) },
                        });
                    }

                    // Extract the ExternalPtr from data1
                    match unsafe { ::miniextendr_api::altrep_data1_as::<#data_ty>(sexp) } {
                        Some(ptr) => Ok(#ref_ident(ptr)),
                        None => Err(::miniextendr_api::SexpTypeError {
                            expected: SEXPTYPE::EXTPTRSXP,
                            actual: unsafe { ::miniextendr_api::ffi::TYPEOF(sexp) },
                        }),
                    }
                }
            }

            impl std::ops::Deref for #ref_ident {
                type Target = #data_ty;

                fn deref(&self) -> &Self::Target {
                    &*self.0
                }
            }

            #[doc = #mut_doc]
            pub struct #mut_ident(::miniextendr_api::externalptr::ExternalPtr<#data_ty>);

            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            impl ::miniextendr_api::TryFromSexp for #mut_ident {
                type Error = ::miniextendr_api::SexpTypeError;

                fn try_from_sexp(sexp: ::miniextendr_api::ffi::SEXP) -> Result<Self, Self::Error> {
                    use ::miniextendr_api::ffi::SEXPTYPE;

                    // Check if it's an ALTREP object (ALTREP returns c_int, not bool)
                    if unsafe { ::miniextendr_api::ffi::ALTREP(sexp) } == 0 {
                        return Err(::miniextendr_api::SexpTypeError {
                            expected: SEXPTYPE::INTSXP, // placeholder - ALTREP check failed
                            actual: unsafe { ::miniextendr_api::ffi::TYPEOF(sexp) },
                        });
                    }

                    // Extract the ExternalPtr from data1
                    match unsafe { ::miniextendr_api::altrep_data1_as::<#data_ty>(sexp) } {
                        Some(ptr) => Ok(#mut_ident(ptr)),
                        None => Err(::miniextendr_api::SexpTypeError {
                            expected: SEXPTYPE::EXTPTRSXP,
                            actual: unsafe { ::miniextendr_api::ffi::TYPEOF(sexp) },
                        }),
                    }
                }
            }

            impl std::ops::Deref for #mut_ident {
                type Target = #data_ty;

                fn deref(&self) -> &Self::Target {
                    &*self.0
                }
            }

            impl std::ops::DerefMut for #mut_ident {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut *self.0
                }
            }
        }
    };

    // Generate doc strings for trait impls
    let base_doc = base_name
        .as_ref()
        .map(|b| b.to_string())
        .unwrap_or_else(|| "inferred".to_string());
    let altrep_class_doc = format!(
        "ALTREP class descriptor for [`{}`] (class: `{}`, pkg: `{}`, base: `{}`).",
        ident, class_name, pkg_name, base_doc
    );
    let method_registrar_doc = format!("Method installer for [`{}`] ALTREP class.", ident);
    let register_altrep_doc = format!("Registration entry point for [`{}`] ALTREP class.", ident);

    let expanded = quote::quote! {
        #input

        // Helper methods for creating ALTREP instances
        #data_helper_impl

        #[doc = #altrep_class_doc]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        impl ::miniextendr_api::altrep::AltrepClass for #ident #ty_generics #where_clause {
            const CLASS_NAME: &'static std::ffi::CStr = #class_cstr;
            const PKG_NAME: &'static std::ffi::CStr = #pkg_cstr;
            const BASE: ::miniextendr_api::altrep::RBase = #base_variant;
            unsafe fn length(x: ::miniextendr_api::ffi::SEXP) -> ::miniextendr_api::ffi::R_xlen_t {
                <#tramp_ty as ::miniextendr_api::altrep_traits::Altrep>::length(x)
            }
        }

        #[doc = #method_registrar_doc]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        impl ::miniextendr_api::altrep_registration::MethodRegistrar for #ident #ty_generics #where_clause {
            unsafe fn install(cls: ::miniextendr_api::ffi::altrep::R_altrep_class_t) {
                use ::miniextendr_api::altrep_bridge as bridge;
                use ::miniextendr_api::ffi::altrep::*;
                // Local helper to reduce boilerplate
                macro_rules! set_if { ($cond:expr, $setter:path, $tramp:expr) => { if $cond { unsafe { $setter(cls, Some($tramp)) } } } }
                // Base: length is ALWAYS required (no HAS_LENGTH check)
                unsafe { R_set_altrep_Length_method(cls, Some(bridge::t_length::<#tramp_ty>)); }
                // Vec-level setters
                set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltVec>::HAS_DATAPTR, R_set_altvec_Dataptr_method, bridge::t_dataptr::<#tramp_ty>);
                set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltVec>::HAS_DATAPTR_OR_NULL, R_set_altvec_Dataptr_or_null_method, bridge::t_dataptr_or_null::<#tramp_ty>);
                set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltVec>::HAS_EXTRACT_SUBSET, R_set_altvec_Extract_subset_method, bridge::t_extract_subset::<#tramp_ty>);
                // Family-specific
                #family_setters
            }
        }

        #[doc = #register_altrep_doc]
        impl ::miniextendr_api::altrep_registration::RegisterAltrep for #ident #ty_generics #where_clause {
            fn get_or_init_class() -> ::miniextendr_api::ffi::altrep::R_altrep_class_t {
                use std::sync::OnceLock;
                static CLASS: OnceLock<::miniextendr_api::ffi::altrep::R_altrep_class_t> = OnceLock::new();
                *CLASS.get_or_init(move || {
                    let cls = unsafe { #make_class };
                    unsafe { <#ident as ::miniextendr_api::altrep_registration::MethodRegistrar>::install(cls); }
                    cls
                })
            }
        }

    };

    expanded.into()
}

/// Generate thread-checked wrappers for R FFI functions.
///
/// Apply this to an `extern "C-unwind"` block to generate checked wrappers
/// that assert we're on the main thread in debug builds.
///
/// **Limitations:**
/// - Variadic functions and statics are passed through unchanged
/// - Only non-variadic functions get checked wrappers
///
/// # Example
///
/// ```ignore
/// #[r_ffi_checked]
/// unsafe extern "C-unwind" {
///     pub fn Rf_ScalarInteger(arg1: i32) -> SEXP;
/// }
/// ```
///
/// Generates:
/// ```ignore
/// unsafe extern "C-unwind" {
///     #[link_name = "Rf_ScalarInteger"]
///     pub fn Rf_ScalarInteger_unchecked(arg1: i32) -> SEXP;
/// }
///
/// #[inline(always)]
/// pub unsafe fn Rf_ScalarInteger(arg1: i32) -> SEXP {
///     debug_assert!(is_r_main_thread(), "Rf_ScalarInteger called from non-main thread");
///     Rf_ScalarInteger_unchecked(arg1)
/// }
/// ```
#[proc_macro_attribute]
pub fn r_ffi_checked(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let foreign_mod = syn::parse_macro_input!(item as syn::ItemForeignMod);

    let abi = &foreign_mod.abi;
    let mut unchecked_items = Vec::new();
    let mut checked_wrappers = Vec::new();

    for item in &foreign_mod.items {
        match item {
            syn::ForeignItem::Fn(fn_item) => {
                let is_variadic = fn_item.sig.variadic.is_some();

                // Check if function already has #[link_name] - if so, pass through unchanged
                let has_link_name = fn_item
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("link_name"));

                if is_variadic || has_link_name {
                    // Pass through variadic functions and functions with explicit link_name unchanged
                    unchecked_items.push(item.clone());
                } else {
                    // Generate checked wrapper for non-variadic functions
                    let vis = &fn_item.vis;
                    let fn_name = &fn_item.sig.ident;
                    let fn_name_str = fn_name.to_string();
                    let unchecked_name = quote::format_ident!("{}_unchecked", fn_name);
                    let inputs = &fn_item.sig.inputs;
                    let output = &fn_item.sig.output;
                    // Filter out link_name attributes (already checked above, but be safe)
                    let attrs: Vec<_> = fn_item
                        .attrs
                        .iter()
                        .filter(|attr| !attr.path().is_ident("link_name"))
                        .collect();

                    // Generate the unchecked FFI binding with #[link_name]
                    let link_name = syn::LitStr::new(&fn_name_str, fn_name.span());
                    let unchecked_fn: syn::ForeignItem = syn::parse_quote! {
                        #(#attrs)*
                        #[link_name = #link_name]
                        #vis fn #unchecked_name(#inputs) #output;
                    };
                    unchecked_items.push(unchecked_fn);

                    // Generate a checked wrapper function
                    let arg_names: Vec<_> = inputs
                        .iter()
                        .filter_map(|arg| {
                            #[allow(clippy::collapsible_if)]
                            if let syn::FnArg::Typed(pat_type) = arg {
                                if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                                    return Some(pat_ident.ident.clone());
                                }
                            }
                            None
                        })
                        .collect();

                    let wrapper = quote::quote! {
                        #(#attrs)*
                        #[inline(always)]
                        #[allow(non_snake_case)]
                        #vis unsafe fn #fn_name(#inputs) #output {
                            #[cfg(debug_assertions)]
                            if !::miniextendr_api::worker::is_r_main_thread() {
                                panic!(concat!("R API `", #fn_name_str, "` called from non-main thread"));
                            }
                            #unchecked_name(#(#arg_names),*)
                        }
                    };
                    checked_wrappers.push(wrapper);
                }
            }
            _ => {
                // Pass through statics and other items unchanged
                unchecked_items.push(item.clone());
            }
        }
    }

    let expanded = quote::quote! {
        unsafe #abi {
            #(#unchecked_items)*
        }

        #(#checked_wrappers)*
    };

    expanded.into()
}

/// Derive macro for implementing `RNative` on a newtype wrapper.
///
/// This allows newtype wrappers around R native types to be used with `Coerce<R>`.
/// The inner type must implement `RNative`.
///
/// # Supported Struct Forms
///
/// Both tuple structs and single-field named structs are supported:
///
/// ```ignore
/// use miniextendr_api::RNative;
///
/// // Tuple struct (most common)
/// #[derive(Clone, Copy, RNative)]
/// struct UserId(i32);
///
/// // Named single-field struct
/// #[derive(Clone, Copy, RNative)]
/// struct Temperature { celsius: f64 }
/// ```
///
/// # Generated Code
///
/// For `struct UserId(i32)`, this generates:
///
/// ```ignore
/// impl RNative for UserId {
///     const SEXP_TYPE: SEXPTYPE = <i32 as RNative>::SEXP_TYPE;
/// }
/// ```
///
/// # Using the Newtype with Coerce
///
/// Once `RNative` is derived, you can implement `Coerce` to/from the newtype:
///
/// ```ignore
/// impl Coerce<UserId> for i32 {
///     fn coerce(self) -> UserId { UserId(self) }
/// }
///
/// let id: UserId = 42.coerce();
/// ```
///
/// # Requirements
///
/// - Must be a newtype struct (exactly one field, tuple or named)
/// - The inner type must implement `RNative` (`i32`, `f64`, `Rboolean`, `u8`, `Rcomplex`, or another derived type)
/// - Should also derive `Copy` (required by `RNative: Copy`)
#[proc_macro_derive(RNative)]
pub fn derive_rnative(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Extract inner type - must be a newtype (single field)
    let inner_ty: syn::Type = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                fields.unnamed.first().unwrap().ty.clone()
            }
            syn::Fields::Named(fields) if fields.named.len() == 1 => {
                fields.named.first().unwrap().ty.clone()
            }
            _ => {
                return syn::Error::new_spanned(
                    name,
                    "#[derive(RNative)] requires a newtype struct with exactly one field",
                )
                .into_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "#[derive(RNative)] only works on structs")
                .into_compile_error()
                .into();
        }
    };

    let expanded = quote::quote! {
        impl #impl_generics ::miniextendr_api::coerce::RNative for #name #ty_generics #where_clause {
            const SEXP_TYPE: ::miniextendr_api::ffi::SEXPTYPE =
                <#inner_ty as ::miniextendr_api::coerce::RNative>::SEXP_TYPE;
        }
    };

    expanded.into()
}

/// Derive macro for implementing `TypedExternal` on a type.
///
/// This makes the type compatible with `ExternalPtr<T>` for storing in R's external pointers.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::TypedExternal;
///
/// #[derive(TypedExternal)]
/// struct MyData {
///     value: i32,
/// }
///
/// // Now you can use ExternalPtr<MyData>
/// let ptr = ExternalPtr::new(MyData { value: 42 });
/// ```
///
/// # Generated Code
///
/// For a type `MyData`, this generates:
///
/// ```ignore
/// impl TypedExternal for MyData {
///     const TYPE_NAME: &'static str = "MyData";
///     const TYPE_NAME_CSTR: &'static [u8] = b"MyData\0";
/// }
/// ```
#[proc_macro_derive(ExternalPtr)]
pub fn derive_external_ptr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Create string literal from type name
    let name_str = name.to_string();
    let name_lit = syn::LitStr::new(&name_str, name.span());
    let name_cstr = syn::LitByteStr::new(format!("{}\0", name_str).as_bytes(), name.span());

    let expanded = quote::quote! {
        impl #impl_generics ::miniextendr_api::externalptr::TypedExternal for #name #ty_generics #where_clause {
            const TYPE_NAME: &'static str = #name_lit;
            const TYPE_NAME_CSTR: &'static [u8] = #name_cstr;
        }
    };

    expanded.into()
}
