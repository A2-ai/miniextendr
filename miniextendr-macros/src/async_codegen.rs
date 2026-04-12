//! Code generation for `#[miniextendr(background)]` functions.
//!
//! Generates three C wrappers and one R env-class wrapper:
//! - `C_fn`: converts args, spawns background thread, returns ExternalPtr<MxAsyncHandle>
//! - `C_fn__is_resolved`: non-blocking status check
//! - `C_fn__value`: blocks until result, downcasts, IntoR conversion
//!
//! The R wrapper creates an environment-class object with `$is_resolved()` and `$value()`.

use proc_macro2::TokenStream;
use quote::quote;

use crate::r_wrapper_builder;

/// Information needed to generate background function code.
pub(crate) struct BackgroundFnInfo<'a> {
    /// The original function identifier (e.g., `slow_compute`)
    pub rust_ident: &'a syn::Ident,
    /// C wrapper identifier (e.g., `C_slow_compute`)
    pub c_ident: &'a syn::Ident,
    /// Function inputs (the parameters)
    pub inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    /// Function return type
    pub output: &'a syn::ReturnType,
    /// Visibility of the original function
    pub vis: &'a syn::Visibility,
    /// Rust parameter identifiers (for calling the function)
    pub rust_inputs: &'a [syn::Ident],
    /// Whether the function is pub (for @export)
    pub is_pub: bool,
    /// Whether to use error_in_r mode
    #[allow(dead_code)]
    pub error_in_r: bool,
    /// Whether to apply strict conversions
    #[allow(dead_code)]
    pub strict: bool,
    /// Whether to apply coercion to all params
    #[allow(dead_code)]
    pub coerce_all: bool,
    /// Roxygen doc tags from the original function
    pub roxygen_tags: &'a [String],
    /// #[cfg(...)] attributes to propagate
    pub cfg_attrs: &'a [syn::Attribute],
    /// Whether internal
    pub internal: bool,
    /// Whether noexport
    pub noexport: bool,
    /// Whether explicit export
    pub export: bool,
}

/// Generate the complete output for a `#[miniextendr(background)]` function.
///
/// Returns `(rust_items, r_wrapper_string)` where:
/// - `rust_items`: TokenStream with original fn + 3 C wrappers + 3 CallMethodDef registrations
/// - `r_wrapper_string`: The R env-class wrapper code
pub(crate) fn generate_background_fn(
    info: &BackgroundFnInfo,
    original_item: &syn::ItemFn,
    conversion_stmts: &[TokenStream],
) -> (TokenStream, String) {
    let rust_ident = info.rust_ident;
    let c_ident = info.c_ident;
    let vis = info.vis;
    let rust_inputs = info.rust_inputs;
    let cfg_attrs = info.cfg_attrs;

    // Build C wrapper parameter list: (__miniextendr_call, arg0, arg1, ...)
    let call_param = syn::Ident::new("__miniextendr_call", proc_macro2::Span::call_site());
    let mut c_params: Vec<TokenStream> = vec![quote!(#call_param: ::miniextendr_api::ffi::SEXP)];
    for arg in info.inputs.iter() {
        if let syn::FnArg::Typed(pt) = arg
            && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
        {
            let mut clean_ident = pat_ident.clone();
            clean_ident.mutability = None;
            clean_ident.by_ref = None;
            let ident = clean_ident;
            c_params.push(quote!(#ident: ::miniextendr_api::ffi::SEXP));
        }
    }

    // Determine the concrete return type for the downcast in __value
    let return_type = match info.output {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, ty) => quote!(#ty),
    };

    // Check if the return type is Result<T, E> for special handling
    let is_result_type = is_result_return(info.output);

    // Generate the boxing expression for the background thread.
    // For Result<T, E>: unwrap in thread, box T on Ok, error string on Err
    // For other types: box directly
    let box_expr = if is_result_type {
        quote! {
            match __bg_raw_result {
                Ok(val) => Ok(Box::new(val) as Box<dyn ::std::any::Any + Send>),
                Err(ref e) => Err(::std::format!("{}", e)),
            }
        }
    } else {
        quote! {
            Ok(Box::new(__bg_raw_result) as Box<dyn ::std::any::Any + Send>)
        }
    };

    // For the value() wrapper, determine the downcast target type
    // If Result<T, E>, the boxed value is T (the Ok variant)
    // Otherwise, it's the full return type
    let (downcast_type, value_return_expr) = if is_result_type {
        let inner_type = extract_result_ok_type(info.output);
        (
            inner_type.clone(),
            quote! {
                ::miniextendr_api::into_r::IntoR::into_sexp(__downcast_val)
            },
        )
    } else if matches!(info.output, syn::ReturnType::Default) {
        (
            quote!(()),
            quote! { unsafe { ::miniextendr_api::ffi::R_NilValue } },
        )
    } else {
        (
            return_type.clone(),
            quote! {
                ::miniextendr_api::into_r::IntoR::into_sexp(__downcast_val)
            },
        )
    };

    let thread_name = format!("mx-bg-{}", rust_ident);

    // ═══════════════════════════════════════════════════════════════════
    // C wrapper 1: C_fn — submit (spawns thread, returns handle)
    // ═══════════════════════════════════════════════════════════════════
    let c_submit_ident = c_ident;
    let c_submit = quote! {
        #(#cfg_attrs)*
        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        #vis extern "C-unwind" fn #c_submit_ident(#(#c_params),*) -> ::miniextendr_api::ffi::SEXP {
            ::miniextendr_api::unwind_protect::with_r_unwind_protect_error_in_r(
                || {
                    #(#conversion_stmts)*

                    let (__mx_handle, __mx_sender) =
                        ::miniextendr_api::async_handle::MxAsyncHandle::new();

                    ::std::thread::Builder::new()
                        .name(#thread_name.into())
                        .spawn(move || {
                            let __bg_panic_result = ::std::panic::catch_unwind(
                                ::std::panic::AssertUnwindSafe(|| #rust_ident(#(#rust_inputs),*))
                            );
                            let __bg_boxed: ::miniextendr_api::async_handle::AsyncResult =
                                match __bg_panic_result {
                                    Ok(__bg_raw_result) => { #box_expr }
                                    Err(__bg_payload) => Err(
                                        ::miniextendr_api::unwind_protect::panic_payload_to_string(
                                            &*__bg_payload,
                                        )
                                    ),
                                };
                            __mx_sender.send(__bg_boxed);
                        })
                        .expect("failed to spawn background thread");

                    ::miniextendr_api::externalptr::ExternalPtr::new(__mx_handle).as_sexp()
                },
                Some(#call_param),
            )
        }
    };

    // ═══════════════════════════════════════════════════════════════════
    // C wrapper 2: C_fn__is_resolved — non-blocking status check
    // ═══════════════════════════════════════════════════════════════════
    let c_is_resolved_name = format!("{}__is_resolved", c_ident);
    let c_is_resolved_ident = syn::Ident::new(&c_is_resolved_name, proc_macro2::Span::call_site());

    let c_is_resolved = quote! {
        #(#cfg_attrs)*
        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        #vis extern "C-unwind" fn #c_is_resolved_ident(
            __miniextendr_call: ::miniextendr_api::ffi::SEXP,
            __mx_handle_sexp: ::miniextendr_api::ffi::SEXP,
        ) -> ::miniextendr_api::ffi::SEXP {
            ::miniextendr_api::unwind_protect::with_r_unwind_protect_error_in_r(
                || {
                    let __ptr = unsafe {
                        ::miniextendr_api::externalptr::ExternalPtr::<
                            ::miniextendr_api::async_handle::MxAsyncHandle,
                        >::wrap_sexp(__mx_handle_sexp).unwrap()
                    };
                    let __resolved = __ptr.as_ref().unwrap().is_resolved();
                    ::miniextendr_api::into_r::IntoR::into_sexp(__resolved)
                },
                Some(__miniextendr_call),
            )
        }
    };

    // ═══════════════════════════════════════════════════════════════════
    // C wrapper 3: C_fn__value — block, downcast, IntoR
    // ═══════════════════════════════════════════════════════════════════
    let c_value_name = format!("{}__value", c_ident);
    let c_value_ident = syn::Ident::new(&c_value_name, proc_macro2::Span::call_site());

    let c_value = quote! {
        #(#cfg_attrs)*
        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        #vis extern "C-unwind" fn #c_value_ident(
            __miniextendr_call: ::miniextendr_api::ffi::SEXP,
            __mx_handle_sexp: ::miniextendr_api::ffi::SEXP,
        ) -> ::miniextendr_api::ffi::SEXP {
            ::miniextendr_api::unwind_protect::with_r_unwind_protect_error_in_r(
                || {
                    let __ptr = unsafe {
                        ::miniextendr_api::externalptr::ExternalPtr::<
                            ::miniextendr_api::async_handle::MxAsyncHandle,
                        >::wrap_sexp(__mx_handle_sexp).unwrap()
                    };
                    match __ptr.as_ref().unwrap().collect_result() {
                        Ok(__boxed) => {
                            let __downcast_val: #downcast_type =
                                *__boxed.downcast::<#downcast_type>()
                                    .expect("type mismatch in async result");
                            #value_return_expr
                        }
                        Err(__msg) => {
                            ::miniextendr_api::error_value::make_rust_error_value(
                                &__msg, "async_err", Some(__miniextendr_call),
                            )
                        }
                    }
                },
                Some(__miniextendr_call),
            )
        }
    };

    // ═══════════════════════════════════════════════════════════════════
    // R_CallMethodDef registrations (3 entries)
    // ═══════════════════════════════════════════════════════════════════
    let submit_def_ident = syn::Ident::new(
        &format!("call_method_def_{}", c_submit_ident),
        proc_macro2::Span::call_site(),
    );
    let is_resolved_def_ident = syn::Ident::new(
        &format!("call_method_def_{}", c_is_resolved_ident),
        proc_macro2::Span::call_site(),
    );
    let value_def_ident = syn::Ident::new(
        &format!("call_method_def_{}", c_value_ident),
        proc_macro2::Span::call_site(),
    );

    // +1 for __miniextendr_call param
    let submit_num_args = info.inputs.len() + 1;
    let submit_num_args_lit =
        syn::LitInt::new(&submit_num_args.to_string(), proc_macro2::Span::call_site());

    let submit_c_name = make_c_name_literal(c_submit_ident);
    let is_resolved_c_name = make_c_name_literal(&c_is_resolved_ident);
    let value_c_name = make_c_name_literal(&c_value_ident);

    // func_ptr types for submit (SEXP per arg + call)
    let submit_ptr_types: Vec<syn::Type> = (0..submit_num_args)
        .map(|_| syn::parse_quote!(::miniextendr_api::ffi::SEXP))
        .collect();

    let registrations = quote! {
        #(#cfg_attrs)*
        #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS)]
        #[linkme(crate = ::miniextendr_api::linkme)]
        #[allow(non_upper_case_globals, non_snake_case)]
        static #submit_def_ident: ::miniextendr_api::ffi::R_CallMethodDef = unsafe {
            ::miniextendr_api::ffi::R_CallMethodDef {
                name: #submit_c_name.as_ptr(),
                fun: Some(::std::mem::transmute::<
                    unsafe extern "C-unwind" fn(#(#submit_ptr_types),*) -> ::miniextendr_api::ffi::SEXP,
                    unsafe extern "C-unwind" fn() -> *mut ::std::os::raw::c_void,
                >(#c_submit_ident)),
                numArgs: #submit_num_args_lit,
            }
        };

        #(#cfg_attrs)*
        #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS)]
        #[linkme(crate = ::miniextendr_api::linkme)]
        #[allow(non_upper_case_globals, non_snake_case)]
        static #is_resolved_def_ident: ::miniextendr_api::ffi::R_CallMethodDef = unsafe {
            ::miniextendr_api::ffi::R_CallMethodDef {
                name: #is_resolved_c_name.as_ptr(),
                fun: Some(::std::mem::transmute::<
                    unsafe extern "C-unwind" fn(
                        ::miniextendr_api::ffi::SEXP,
                        ::miniextendr_api::ffi::SEXP,
                    ) -> ::miniextendr_api::ffi::SEXP,
                    unsafe extern "C-unwind" fn() -> *mut ::std::os::raw::c_void,
                >(#c_is_resolved_ident)),
                numArgs: 2,
            }
        };

        #(#cfg_attrs)*
        #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS)]
        #[linkme(crate = ::miniextendr_api::linkme)]
        #[allow(non_upper_case_globals, non_snake_case)]
        static #value_def_ident: ::miniextendr_api::ffi::R_CallMethodDef = unsafe {
            ::miniextendr_api::ffi::R_CallMethodDef {
                name: #value_c_name.as_ptr(),
                fun: Some(::std::mem::transmute::<
                    unsafe extern "C-unwind" fn(
                        ::miniextendr_api::ffi::SEXP,
                        ::miniextendr_api::ffi::SEXP,
                    ) -> ::miniextendr_api::ffi::SEXP,
                    unsafe extern "C-unwind" fn() -> *mut ::std::os::raw::c_void,
                >(#c_value_ident)),
                numArgs: 2,
            }
        };
    };

    // ═══════════════════════════════════════════════════════════════════
    // R wrapper (env-class handle)
    // ═══════════════════════════════════════════════════════════════════
    let r_wrapper_string = generate_r_wrapper(info, &c_is_resolved_name, &c_value_name);

    // ═══════════════════════════════════════════════════════════════════
    // R wrapper registration
    // ═══════════════════════════════════════════════════════════════════
    let r_wrapper_generator_ident = syn::Ident::new(
        &format!("R_WRAPPER_{}", rust_ident.to_string().to_uppercase()),
        proc_macro2::Span::call_site(),
    );
    let r_wrapper_str = crate::r_wrapper_raw_literal(&r_wrapper_string);
    let source_start = rust_ident.span().start();
    let source_line_lit = syn::LitInt::new(&source_start.line.to_string(), rust_ident.span());
    let source_col_lit =
        syn::LitInt::new(&(source_start.column + 1).to_string(), rust_ident.span());

    let r_wrapper_reg = quote! {
        #(#cfg_attrs)*
        #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_R_WRAPPERS)]
        #[linkme(crate = ::miniextendr_api::linkme)]
        #[allow(non_upper_case_globals, non_snake_case)]
        static #r_wrapper_generator_ident: ::miniextendr_api::registry::RWrapperEntry =
            ::miniextendr_api::registry::RWrapperEntry {
                priority: ::miniextendr_api::registry::RWrapperPriority::Function,
                source_file: file!(),
                content: concat!(
                    "# Generated from Rust fn `",
                    stringify!(#rust_ident),
                    "` (",
                    file!(),
                    ":",
                    #source_line_lit,
                    ":",
                    #source_col_lit,
                    ")",
                    #r_wrapper_str
                ),
            };
    };

    // Assemble everything
    let mut stripped_item = original_item.clone();
    stripped_item
        .attrs
        .retain(|attr| !attr.path().is_ident("miniextendr"));

    let tokens = quote! {
        #stripped_item
        #c_submit
        #c_is_resolved
        #c_value
        #registrations
        #r_wrapper_reg
    };

    (tokens, r_wrapper_string)
}

/// Generate the R env-class wrapper for a background function.
fn generate_r_wrapper(
    info: &BackgroundFnInfo,
    c_is_resolved_name: &str,
    c_value_name: &str,
) -> String {
    let rust_ident = info.rust_ident;
    let c_ident = info.c_ident;

    // Build R formal parameters
    let arg_builder = r_wrapper_builder::RArgumentBuilder::new(info.inputs);
    let r_formals = arg_builder.build_formals();
    let mut r_call_args = arg_builder.build_call_args_vec();
    r_call_args.insert(0, ".call = match.call()".to_string());
    let call_args_joined = r_call_args.join(", ");

    // Roxygen tags
    let roxygen = if !info.roxygen_tags.is_empty() {
        crate::roxygen::format_roxygen_tags(info.roxygen_tags)
    } else {
        String::new()
    };

    let source_comment = format!(
        "#' @source Generated by miniextendr from Rust fn `{}` (background)\n",
        rust_ident
    );

    let export_comment = if (info.is_pub || info.export) && !info.internal && !info.noexport {
        "#' @export\n"
    } else {
        ""
    };

    let internal_comment = if info.internal {
        "#' @keywords internal\n"
    } else {
        ""
    };

    let r_fn_name = rust_ident.to_string();

    format!(
        r#"{roxygen}{source_comment}{internal_comment}{export_comment}{r_fn_name} <- function({formals}) {{
  .ptr <- .Call({c_ident}, {call_args})
  if (inherits(.ptr, "rust_error_value")) stop(.ptr)
  .e <- new.env(parent = emptyenv())
  .e$.ptr <- .ptr
  .e$is_resolved <- function() {{
    .Call({c_is_resolved}, .call = match.call(), .e$.ptr)
  }}
  .e$value <- function() {{
    .val <- .Call({c_value}, .call = match.call(), .e$.ptr)
    if (inherits(.val, "rust_error_value")) stop(.val)
    .val
  }}
  class(.e) <- "mx_async_handle"
  .e
}}"#,
        roxygen = roxygen,
        source_comment = source_comment,
        internal_comment = internal_comment,
        export_comment = export_comment,
        r_fn_name = r_fn_name,
        formals = r_formals,
        c_ident = c_ident,
        call_args = call_args_joined,
        c_is_resolved = c_is_resolved_name,
        c_value = c_value_name,
    )
}

/// Check if a return type is `Result<T, E>`.
fn is_result_return(output: &syn::ReturnType) -> bool {
    if let syn::ReturnType::Type(_, ty) = output
        && let syn::Type::Path(type_path) = ty.as_ref()
        && let Some(seg) = type_path.path.segments.last()
    {
        return seg.ident == "Result";
    }
    false
}

/// Extract the `T` from `Result<T, E>`.
fn extract_result_ok_type(output: &syn::ReturnType) -> TokenStream {
    if let syn::ReturnType::Type(_, ty) = output
        && let syn::Type::Path(type_path) = ty.as_ref()
        && let Some(seg) = type_path.path.segments.last()
        && seg.ident == "Result"
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(ok_type)) = args.args.first()
    {
        return quote!(#ok_type);
    }
    // Fallback: use the full return type
    match output {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, ty) => quote!(#ty),
    }
}

/// Create a `syn::LitCStr` from an identifier.
fn make_c_name_literal(ident: &syn::Ident) -> syn::LitCStr {
    syn::LitCStr::new(
        std::ffi::CString::new(ident.to_string())
            .expect("valid C string")
            .as_c_str(),
        proc_macro2::Span::call_site(),
    )
}
