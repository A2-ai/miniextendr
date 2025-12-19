// miniextendr-macros procedural macros

mod altrep;
mod miniextendr_fn;
use crate::miniextendr_fn::{CoercionMapping, MiniextendrFnAttrs, MiniextendrFunctionParsed};
mod miniextendr_module;
use crate::miniextendr_module::MiniextendrModule;

/// Identifier for the generated `const fn` returning an `R_CallMethodDef`.
///
/// This must remain consistent between the attribute macro (which defines the symbol)
/// and the module macro (which references it).
pub(crate) fn call_method_def_ident_for(rust_ident: &syn::Ident) -> syn::Ident {
    quote::format_ident!("call_method_def_{rust_ident}")
}

/// Identifier for the generated `const &str` holding the R wrapper source code.
///
/// This must remain consistent between the attribute macro (which defines the symbol)
/// and the module macro (which references it).
pub(crate) fn r_wrapper_const_ident_for(rust_ident: &syn::Ident) -> syn::Ident {
    let rust_ident_upper = rust_ident.to_string().to_uppercase();
    quote::format_ident!("R_WRAPPER_{rust_ident_upper}")
}

/// Normalize a Rust parameter identifier into an R argument identifier.
///
/// The generated R wrapper uses this to avoid exporting leading underscore names.
/// - `__foo` → `private__foo`
/// - `_foo` → `unused_foo`
fn normalize_r_arg_ident(rust_ident: &syn::Ident) -> syn::Ident {
    let mut arg_name = rust_ident.to_string();
    if arg_name.starts_with("__") {
        arg_name.insert_str(0, "private");
    } else if arg_name.starts_with('_') {
        arg_name.insert_str(0, "unused");
    }
    syn::Ident::new(&arg_name, rust_ident.span())
}

/// Extract `#[cfg(...)]` attributes from a list of attributes.
///
/// These should be propagated to generated items so they are conditionally
/// compiled along with the original function.
fn extract_cfg_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg"))
        .cloned()
        .collect()
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

#[inline]
fn is_sexp_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p
        .path
        .segments
        .last()
        .map(|s| s.ident == "SEXP")
        .unwrap_or(false))
}

#[proc_macro_attribute]
pub fn miniextendr(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // If not a function, delegate to ALTREP path (allow structs/enums)
    if syn::parse::<syn::ItemFn>(item.clone()).is_err() {
        return altrep::expand_altrep_struct(attr, item);
    }

    let MiniextendrFnAttrs {
        force_main_thread,
        force_invisible,
        check_interrupt,
        coerce_all,
    } = syn::parse_macro_input!(attr as MiniextendrFnAttrs);

    let mut parsed = syn::parse_macro_input!(item as MiniextendrFunctionParsed);
    parsed.add_track_caller_if_needed();
    parsed.add_inline_never_if_needed();

    // Extract commonly used values
    let uses_internal_c_wrapper = parsed.uses_internal_c_wrapper();
    let call_method_def = parsed.call_method_def_ident();
    let c_ident = parsed.c_wrapper_ident();
    let r_wrapper_generator = parsed.r_wrapper_const_ident();

    use quote::ToTokens;
    use syn::spanned::Spanned;

    // Extract references to parsed components
    let rust_ident = parsed.ident();
    let inputs = parsed.inputs();
    let output = parsed.output();
    let abi = parsed.abi();
    let attrs = parsed.attrs();
    let vis = parsed.vis();
    let generics = parsed.generics();
    let has_dots = parsed.has_dots();
    let mut named_dots = parsed.named_dots().cloned();

    let rust_arg_count = inputs.len();
    let registered_arg_count = if uses_internal_c_wrapper {
        rust_arg_count + 1
    } else {
        rust_arg_count
    };
    let num_args = syn::LitInt::new(&registered_arg_count.to_string(), inputs.span());

    // name of the C-wrapper
    let c_ident_name = syn::LitCStr::new(
        std::ffi::CString::new(c_ident.to_string())
            .expect("couldn't create a C-string for the C wrapper name")
            .as_c_str(),
        rust_ident.span(),
    );
    // registration of the C-wrapper
    // these are needed to transmute fn-item to fn-pointer correctly.
    let mut func_ptr_def: Vec<syn::Type> = Vec::new();
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
                } else if is_slice {
                    // Slice references use TryFromSexp (backed by DATAPTR_RO).
                    closure_statements.push(quote::quote! {
                        let #ident = ::miniextendr_api::TryFromSexp::try_from_sexp(#ident).unwrap();
                    });
                } else if is_str {
                    // `&str` parameters are decoded to an owned `String` first (via TryFromSexp),
                    // then borrowed as `&str` for the Rust call. This avoids returning a `'static`
                    // borrow into R memory and also allows UTF-8 translation when needed.
                    let storage_ident = quote::format_ident!("__miniextendr_{}_string", ident);
                    let mutability = if pat_ident.mutability.is_some() {
                        quote::quote!(mut)
                    } else {
                        quote::quote!()
                    };
                    closure_statements.push(quote::quote! {
                        let #storage_ident: String = ::miniextendr_api::TryFromSexp::try_from_sexp(#ident).unwrap();
                    });
                    closure_statements.push(quote::quote! {
                        let #mutability #ident: &str = #storage_ident.as_str();
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
                // - has_coerce_attr: #[miniextendr(coerce)] on individual param
                let param_name = ident.to_string();
                let should_coerce = coerce_all || parsed.has_coerce_attr(&param_name);
                let coercion_mapping = if should_coerce {
                    CoercionMapping::from_type(pat_type.ty.as_ref())
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
                    Some(CoercionMapping::Vec {
                        r_native_elem,
                        target_elem,
                    }) => {
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

            // -> Option<...> cases
            syn::Type::Path(p)
                if p.path.segments.last().map(|s| &s.ident)
                    == Some(&syn::Ident::new("Option", p.path.span())) =>
            {
                let seg = p.path.segments.last().unwrap();
                let inner_ty = first_type_argument(seg);
                let is_unit_inner = inner_ty
                    .is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));
                let is_sexp_inner = inner_ty.is_some_and(is_sexp_type);

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
                    if is_sexp_inner {
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
                let ok_is_unit =
                    ok_ty.is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));
                let ok_is_sexp = ok_ty.is_some_and(is_sexp_type);

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
                    if ok_is_sexp {
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

    // Use worker strategy by default for functions that don't return raw SEXP.
    // Worker thread provides proper panic catching with destructor cleanup.
    // Functions returning raw SEXP or taking Dots must stay on main thread (SEXP/Dots aren't Send).
    // Use #[miniextendr(unsafe(main_thread))] to force main thread for functions that call R APIs internally.
    // check_interrupt also requires main thread since R_CheckUserInterrupt must be called there.
    // Note: ExternalPtr<T> returning functions use worker thread - if they call R APIs internally,
    // the FFI wrapper's debug check will catch it and users should add #[miniextendr(unsafe(main_thread))].
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
        *named = normalize_r_arg_ident(named);
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
            syn::Pat::Ident(pat_ident) => normalize_r_arg_ident(&pat_ident.ident),
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
    let export_comment = if matches!(vis, syn::Visibility::Public(_)) {
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

    // endregion

    let abi = abi
        .cloned()
        .unwrap_or_else(|| syn::parse_quote!(extern "C-unwind"));

    // Extract cfg attributes to apply to generated items
    let cfg_attrs = extract_cfg_attrs(parsed.attrs());

    // Generate doc strings with links
    let r_wrapper_doc = format!(
        "R wrapper code for [`{}`], calls [`{}`].",
        rust_ident, c_ident
    );
    let call_method_def_doc = format!(
        "R call method definition for [`{}`] (C wrapper: [`{}`]).",
        rust_ident, c_ident
    );

    // Get the normalized item for output
    let original_item = parsed.item();

    let expanded: proc_macro::TokenStream = quote::quote! {
        // rust function!
        #original_item

        // C wrapper
        #(#cfg_attrs)*
        #c_wrapper

        // R wrapper
        #(#cfg_attrs)*
        #[doc = #r_wrapper_doc]
        const #r_wrapper_generator: &str = #r_wrapper_str;

        // registration of C wrapper in R
        #(#cfg_attrs)*
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

/// Register functions and ALTREP types with R's dynamic symbol registration.
///
/// This macro generates the `R_init_<module>_miniextendr` entrypoint that R calls
/// when loading the shared library.
///
/// # Syntax
///
/// ```ignore
/// miniextendr_module! {
///     mod mymodule;
///
///     // Regular Rust functions (generates safe R wrapper)
///     fn my_function;
///
///     // Raw C ABI functions (R wrapper prefixed with `unsafe_`)
///     extern "C-unwind" fn C_my_raw_function;
///
///     // ALTREP types (registers the class with R)
///     struct MyAltrepClass;
///
///     // Re-export from submodules
///     use submodule;
/// }
/// ```
///
/// # Function Registration
///
/// ## Regular functions (`fn`)
///
/// For functions defined with `#[miniextendr]` that have a Rust signature:
/// - C symbol: `C_<name>` (auto-generated wrapper)
/// - R wrapper: `<name>()` (safe, with type conversion)
///
/// ## Extern functions (`extern "C-unwind" fn`)
///
/// For raw C ABI functions defined with `#[miniextendr]` and `extern "C-unwind"`:
/// - C symbol: The function name you provided (e.g., `C_my_function`)
/// - R wrapper: `unsafe_<name>()` (prefixed to indicate bypassed safety)
///
/// The `unsafe_` prefix signals to R users that these functions:
/// 1. Run directly on R's thread (no worker thread isolation)
/// 2. May not have proper panic handling
/// 3. Don't perform automatic type conversion
///
/// # ALTREP Registration
///
/// Structs listed are registered as ALTREP classes during `R_init_*`.
/// The struct must implement the appropriate ALTREP traits.
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// fn add(a: i32, b: i32) -> i32 { a + b }
///
/// #[miniextendr]
/// #[unsafe(no_mangle)]
/// extern "C-unwind" fn C_fast_add(a: SEXP, b: SEXP) -> SEXP { /* ... */ }
///
/// miniextendr_module! {
///     mod mypackage;
///     fn add;                         // R: add(a, b)
///     extern "C-unwind" fn C_fast_add; // R: unsafe_C_fast_add()
/// }
/// ```
// TODO: Currently, miniextendr_module does not distinguish between
// `extern "C-unwind" fn` and `fn` items.. they are treated alike.
#[proc_macro]
pub fn miniextendr_module(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_module = syn::parse_macro_input!(item as MiniextendrModule);

    let module = &parsed_module.module_name.ident;
    let module_entrypoint_ident = quote::format_ident!("R_init_{module}_miniextendr");
    let call_entries: Vec<syn::Expr> = parsed_module
        .functions
        .iter()
        .map(|f| {
            let call_method_def = f.call_method_def_ident();
            syn::parse_quote!(#call_method_def())
        })
        .collect();
    let call_entries_len = call_entries.len();

    // Generate ALTREP registrations for struct items (if they implement RegisterAltrep)
    let altrep_regs: Vec<syn::Expr> = parsed_module
        .structs
        .iter()
        .map(|s| {
            let ty = &s.ident;
            syn::parse_quote!(<#ty as ::miniextendr_api::altrep_registration::RegisterAltrep>::get_or_init_class())
        })
        .collect();

    // call the R_init from all the submodules (given by `use`)
    let use_other_modules = parsed_module
        .uses
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let use_module_init = quote::format_ident!("R_init_{use_module_ident}_miniextendr");
            syn::parse_quote!(#use_module_ident::#use_module_init(dll))
        })
        .collect::<Vec<syn::Expr>>();

    // region: r wrapper generation in `mod`

    let r_wrapper_generators: Vec<syn::Expr> = parsed_module
        .functions
        .iter()
        .map(|x| {
            let r_wrapper_const = x.r_wrapper_const_ident();
            syn::parse_quote!(#r_wrapper_const)
        })
        .collect();
    let r_wrappers_use_other_modules = parsed_module
        .uses
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
    let fn_links: Vec<String> = parsed_module
        .functions
        .iter()
        .map(|f| format!("[`{}`]", f.ident))
        .collect();
    let struct_links: Vec<String> = parsed_module
        .structs
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
                ::miniextendr_api::ffi::R_registerRoutines_unchecked(
                    dll,
                    std::ptr::null(),
                    CALL_ENTRIES.as_ptr(),
                    std::ptr::null(),
                    std::ptr::null()
                );
                // R_useDynamicSymbols and R_forceSymbols are called in entrypoint.c
            }
        }
    }
    .into()
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

/// Derive macro for implementing `RNativeType` on a newtype wrapper.
///
/// This allows newtype wrappers around R native types to work with `Vec<T>`,
/// `&[T]` conversions and the `Coerce<R>` traits.
/// The inner type must implement `RNativeType`.
///
/// # Supported Struct Forms
///
/// Both tuple structs and single-field named structs are supported:
///
/// ```ignore
/// use miniextendr_api::RNativeType;
///
/// // Tuple struct (most common)
/// #[derive(Clone, Copy, RNativeType)]
/// struct UserId(i32);
///
/// // Named single-field struct
/// #[derive(Clone, Copy, RNativeType)]
/// struct Temperature { celsius: f64 }
/// ```
///
/// # Generated Code
///
/// For `struct UserId(i32)`, this generates:
///
/// ```ignore
/// impl RNativeType for UserId {
///     const SEXP_TYPE: SEXPTYPE = <i32 as RNativeType>::SEXP_TYPE;
/// }
/// ```
///
/// # Using the Newtype with Coerce
///
/// Once `RNativeType` is derived, you can implement `Coerce` to/from the newtype:
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
/// - The inner type must implement `RNativeType` (`i32`, `f64`, `RLogical`, `u8`, `Rcomplex`)
/// - Should also derive `Copy` (required by `RNativeType: Copy`)
#[proc_macro_derive(RNativeType)]
pub fn derive_rnative_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
                    "#[derive(RNativeType)] requires a newtype struct with exactly one field",
                )
                .into_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "#[derive(RNativeType)] only works on structs")
                .into_compile_error()
                .into();
        }
    };

    let expanded = quote::quote! {
        impl #impl_generics ::miniextendr_api::ffi::RNativeType for #name #ty_generics #where_clause {
            const SEXP_TYPE: ::miniextendr_api::ffi::SEXPTYPE =
                <#inner_ty as ::miniextendr_api::ffi::RNativeType>::SEXP_TYPE;
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

#[cfg(test)]
mod tests;
