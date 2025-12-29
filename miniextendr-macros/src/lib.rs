// miniextendr-macros procedural macros

mod altrep;
mod c_wrapper_builder;
mod miniextendr_fn;
use crate::miniextendr_fn::{MiniextendrFnAttrs, MiniextendrFunctionParsed};
mod miniextendr_impl;
mod miniextendr_module;
use crate::miniextendr_module::MiniextendrModule;
mod r_wrapper_builder;
/// Builder utilities for formatting R wrapper arguments and calls.
pub(crate) use r_wrapper_builder::RArgumentBuilder;
mod rust_conversion_builder;
/// Helper for generating Rust→R conversion code for return values.
pub(crate) use rust_conversion_builder::RustConversionBuilder;
mod method_return_builder;
/// Helpers for shaping method return handling (R vs Rust wrapper code).
pub(crate) use method_return_builder::{MethodReturnBuilder, ReturnStrategy};
mod altrep_derive;
mod list_derive;
mod r_class_formatter;
mod return_type_analysis;
mod roxygen;

// Trait ABI support modules
mod externalptr_derive;
mod miniextendr_impl_trait;
mod miniextendr_trait;

/// Identifier for the generated `const` `R_CallMethodDef` value.
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

// normalize_r_arg_ident is now provided by r_wrapper_builder module

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

/// Export Rust items to R.
///
/// `#[miniextendr]` can be applied to:
/// - `fn` items (generate C + R wrappers)
/// - `impl` blocks (generate R class methods)
/// - `trait` items (generate trait ABI metadata)
/// - ALTREP wrapper structs (generate `RegisterAltrep` impls)
///
/// # Functions
///
/// ```ignore
/// use miniextendr_api::{miniextendr, miniextendr_module};
///
/// #[miniextendr]
/// fn add(a: i32, b: i32) -> i32 { a + b }
///
/// miniextendr_module! {
///     mod mypkg;
///     fn add;
/// }
/// ```
///
/// This produces a C wrapper `C_add` and an R wrapper `add()`.
///
/// ## `extern "C-unwind"`
///
/// If the function is declared `extern "C-unwind"` and exported with
/// `#[no_mangle]` (2021), `#[unsafe(no_mangle)]` (2024), or `#[export_name = "..."]`,
/// the function itself is the C symbol and the R wrapper is prefixed with
/// `unsafe_` to signal bypassed safety (no worker isolation or conversion).
///
/// ## Variadics
///
/// Use `...` as the last argument. The Rust parameter becomes `&Dots` and is
/// renamed to `_dots` in the generated R wrapper.
///
/// ## Attributes
///
/// - `#[miniextendr(unsafe(main_thread))]`
/// - `#[miniextendr(invisible)]` / `#[miniextendr(visible)]`
/// - `#[miniextendr(check_interrupt)]`
/// - `#[miniextendr(coerce)]` (also usable per-parameter)
///
/// # Impl blocks (class systems)
///
/// Apply `#[miniextendr(env|r6|s7|s3|s4)]` to an `impl Type` block and list
/// `impl Type;` in `miniextendr_module!`.
///
/// # Traits (ABI)
///
/// Apply `#[miniextendr]` to a trait to generate ABI metadata, then use
/// `#[miniextendr] impl Trait for Type` and list `impl Trait for Type;` in
/// `miniextendr_module!`.
///
/// # ALTREP
///
/// Apply `#[miniextendr(class = "...", pkg = "...", base = "...")]` to a
/// one-field wrapper struct and list `struct Type;` in `miniextendr_module!`.
#[proc_macro_attribute]
pub fn miniextendr(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Try to parse as function first
    if syn::parse::<syn::ItemFn>(item.clone()).is_ok() {
        // Continue with function handling below
    } else if syn::parse::<syn::ItemImpl>(item.clone()).is_ok() {
        // Delegate to impl block parser
        return miniextendr_impl::expand_impl(attr, item);
    } else if syn::parse::<syn::ItemTrait>(item.clone()).is_ok() {
        // Delegate to trait ABI generator
        return miniextendr_trait::expand_trait(attr, item);
    } else {
        // Delegate to ALTREP path (structs/enums)
        return altrep::expand_altrep_struct(attr, item);
    }

    let MiniextendrFnAttrs {
        force_main_thread,
        force_worker,
        force_invisible,
        check_interrupt,
        coerce_all,
        rng,
        return_pref,
    } = syn::parse_macro_input!(attr as MiniextendrFnAttrs);

    let mut parsed = syn::parse_macro_input!(item as MiniextendrFunctionParsed);

    // Validate: reject generic functions (extern "C-unwind" + #[no_mangle] incompatible with generics)
    if !parsed.item().sig.generics.params.is_empty() {
        let err = syn::Error::new_spanned(
            &parsed.item().sig.generics,
            "#[miniextendr] functions cannot have generic type parameters. \
             Generic functions are incompatible with `extern \"C-unwind\"` and `#[no_mangle]` \
             required for R FFI. Consider using trait objects or monomorphization instead.",
        );
        return err.into_compile_error().into();
    }

    parsed.add_track_caller_if_needed();
    parsed.add_inline_never_if_needed();

    // Extract commonly used values
    let uses_internal_c_wrapper = parsed.uses_internal_c_wrapper();
    let call_method_def = parsed.call_method_def_ident();
    let c_ident = parsed.c_wrapper_ident();
    let r_wrapper_generator = parsed.r_wrapper_const_ident();

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
    let named_dots = parsed.named_dots().cloned();

    // Check for @title/@description conflicts with implicit values (doc-lint feature)
    let doc_lint_warnings = crate::roxygen::doc_conflict_warnings(attrs, rust_ident.span());

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

    // Hygiene: Use call_site() for internal variable names that should be visible to
    // procedural macro machinery but not create unhygienic references to user code.
    // call_site() = span of the macro invocation (#[miniextendr])
    let call_param_ident = syn::Ident::new("__miniextendr_call", proc_macro2::Span::call_site());
    let mut c_wrapper_inputs: Vec<_> = Vec::new();
    if uses_internal_c_wrapper {
        c_wrapper_inputs.push(syn::parse_quote!(#call_param_ident: ::miniextendr_api::ffi::SEXP));
    }
    c_wrapper_inputs.extend(inputs.clone().into_pairs().map(|pair| {
        let arg = pair.value();
        match arg {
            syn::FnArg::Receiver(receiver) => {
                syn::Error::new(
                    receiver.span(),
                    "self parameter not allowed in standalone functions; \
                     use #[miniextendr(env|r6|s3|s4|s7)] on impl blocks instead"
                ).to_compile_error()
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
    // Build conversion builder with coercion settings
    let mut conversion_builder = RustConversionBuilder::new();
    if coerce_all {
        conversion_builder = conversion_builder.with_coerce_all();
    }
    for input in inputs.iter() {
        if let syn::FnArg::Typed(pt) = input
            && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
        {
            let param_name = pat_ident.ident.to_string();
            if parsed.has_coerce_attr(&param_name) {
                conversion_builder = conversion_builder.with_coerce_param(param_name);
            }
        }
    }

    // Generate conversion statements
    let closure_statements = conversion_builder.build_conversions(inputs, &rust_inputs);

    // Hygiene: Use mixed_site() for internal variables that need to reference both
    // macro-generated items (quote!) and user-provided items from the original function.
    // mixed_site() = allows capturing both hygiene contexts for cross-context references.
    let rust_result_ident =
        syn::Ident::new("__miniextendr_rust_result", proc_macro2::Span::mixed_site());

    // Analyze return type to determine:
    // - Whether it returns SEXP (affects thread strategy)
    // - Whether result should be invisible
    // - How to convert Rust → SEXP
    // - Post-call processing (unwrap Option/Result)
    let return_analysis = return_type_analysis::analyze_return_type(
        output,
        &rust_result_ident,
        rust_ident,
        return_pref,
    );

    let returns_sexp = return_analysis.returns_sexp;
    let is_invisible_return_type = return_analysis.is_invisible;
    let return_expression = return_analysis.return_expression;
    let post_call_statements = return_analysis.post_call_statements;

    // Apply explicit visibility override from #[miniextendr(invisible)] or #[miniextendr(visible)]
    let is_invisible_return_type = force_invisible.unwrap_or(is_invisible_return_type);

    // Check if any input parameter is SEXP (not Send, must stay on main thread)
    let has_sexp_inputs = inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            is_sexp_type(pat_type.ty.as_ref())
        } else {
            false
        }
    });

    // ═══════════════════════════════════════════════════════════════════════════
    // Thread Strategy Selection
    // ═══════════════════════════════════════════════════════════════════════════
    //
    // miniextendr supports two execution strategies:
    //
    // 1. **Main Thread Strategy** (with_r_unwind_protect)
    //    - All code runs on R's main thread
    //    - Required when SEXP types are involved (not Send)
    //    - Required for R API calls (Rf_*, R_*)
    //    - Panic handling via R_UnwindProtect (Rust destructors run correctly)
    //    - Use cases:
    //      * Functions returning SEXP
    //      * Functions taking SEXP parameters
    //      * Functions using Dots (variadic args)
    //      * Functions with #[miniextendr(unsafe(main_thread))]
    //      * Functions with check_interrupt (R_CheckUserInterrupt)
    //
    // 2. **Worker Thread Strategy** (run_on_worker + catch_unwind)
    //    - Argument conversion on main thread (SEXP → Rust types)
    //    - Function execution on dedicated worker thread (clean panic isolation)
    //    - Result conversion on main thread (Rust types → SEXP)
    //    - Panic handling via catch_unwind (prevents unwinding across FFI boundary)
    //    - Benefits:
    //      * Clean panic handling with proper destructors
    //      * Isolates user code from R's execution context
    //      * Catches panics from both user code and conversions
    //    - Limitations:
    //      * Cannot call R APIs directly (use ExternalPtr + main_thread if needed)
    //      * All parameters and return types must be Send
    //
    // Default: Worker thread (safer, cleaner panic handling)
    // Override: Use main thread when SEXP types or R APIs are involved
    //
    // Note: ExternalPtr<T> returning functions use worker thread by default.
    // If they internally call R APIs, the debug FFI check will catch it at runtime.
    // Users should add #[miniextendr(unsafe(main_thread))] in that case.
    //
    // Thread strategy:
    // - force_worker overrides the default, but cannot override hard requirements
    // - Hard requirements for main thread: returns_sexp, has_sexp_inputs, has_dots, check_interrupt
    // - force_main_thread is a user request (can be combined with force_worker, but main_thread wins)
    let requires_main_thread = returns_sexp || has_sexp_inputs || has_dots || check_interrupt;
    let use_main_thread = requires_main_thread || (force_main_thread && !force_worker);

    // Suppress unused variable warning when force_worker doesn't affect strategy
    let _ = force_worker;
    // RNG state management tokens
    let (rng_get, rng_put) = if rng {
        (
            quote::quote! { unsafe { ::miniextendr_api::ffi::GetRNGstate(); } },
            quote::quote! { unsafe { ::miniextendr_api::ffi::PutRNGstate(); } },
        )
    } else {
        (
            proc_macro2::TokenStream::new(),
            proc_macro2::TokenStream::new(),
        )
    };

    let c_wrapper = if abi.is_some() {
        proc_macro2::TokenStream::new()
    } else if use_main_thread {
        // SEXP-returning or Dots-taking functions: use with_r_unwind_protect on main thread
        let c_wrapper_doc = format!(
            "C wrapper for [`{}`] (main thread). See [`{}`] for R wrapper.",
            rust_ident, r_wrapper_generator
        );
        if rng {
            // RNG variant: wrap in catch_unwind so we can call PutRNGstate before error handling
            quote::quote! {
                #[doc = #c_wrapper_doc]
                #[unsafe(no_mangle)]
                #vis extern "C-unwind" fn #c_ident #generics(#(#c_wrapper_inputs),*) -> ::miniextendr_api::ffi::SEXP {
                    #rng_get
                    let __result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
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
                    }));
                    #rng_put
                    match __result {
                        Ok(sexp) => sexp,
                        Err(payload) => ::std::panic::resume_unwind(payload),
                    }
                }
            }
        } else {
            // Non-RNG variant: direct call to with_r_unwind_protect
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
        let c_wrapper_doc = format!(
            "C wrapper for [`{}`] (worker thread). See [`{}`] for R wrapper.",
            rust_ident, r_wrapper_generator
        );
        quote::quote! {
            #[doc = #c_wrapper_doc]
            #[unsafe(no_mangle)]
            #vis extern "C-unwind" fn #c_ident #generics(#(#c_wrapper_inputs),*) -> ::miniextendr_api::ffi::SEXP {
                #rng_get
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
                // PutRNGstate runs after catch_unwind, before error conversion
                #rng_put
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
        // check that #[no_mangle] or #[unsafe(no_mangle)] or #[export_name] is present!
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

        let has_export_name = attrs.iter().any(|attr| attr.path().is_ident("export_name"));

        if !has_no_mangle && !has_export_name {
            return syn::Error::new(
                attrs
                    .first()
                    .map(|attr| attr.span())
                    .unwrap_or_else(|| abi.span()),
                "missing #[no_mangle] (edition 2021), #[unsafe(no_mangle)] (edition 2024), or #[export_name = \"...\"]",
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

        // Validate all input types are SEXP for extern "C-unwind" functions.
        // R's .Call interface passes all arguments as SEXP, so accepting other types is UB.
        // Also reject variadic (...) signatures which are not valid for .Call.
        for input in inputs.iter() {
            match input {
                syn::FnArg::Receiver(recv) => {
                    return syn::Error::new_spanned(
                        recv,
                        "extern functions cannot have self parameter",
                    )
                    .into_compile_error()
                    .into();
                }
                syn::FnArg::Typed(pat_type) => {
                    // Check if this is a variadic pattern (...)
                    if let syn::Pat::Rest(_) = pat_type.pat.as_ref() {
                        return syn::Error::new_spanned(
                            pat_type,
                            "extern functions cannot use variadic (...) - .Call passes fixed arguments",
                        )
                        .into_compile_error()
                        .into();
                    }

                    // Validate type is SEXP
                    let is_sexp = match pat_type.ty.as_ref() {
                        syn::Type::Path(type_path) => type_path
                            .path
                            .segments
                            .last()
                            .is_some_and(|seg| seg.ident == "SEXP"),
                        _ => false,
                    };

                    if !is_sexp {
                        return syn::Error::new_spanned(
                            &pat_type.ty,
                            "extern function parameters must be SEXP - .Call passes all arguments as SEXP",
                        )
                        .into_compile_error()
                        .into();
                    }
                }
            }
        }
    }

    // region: R wrappers generation in `fn`
    // Build R formal parameters and call arguments using shared builder
    let mut arg_builder = RArgumentBuilder::new(inputs);
    if has_dots {
        arg_builder = arg_builder.with_dots(named_dots.map(|id| id.to_string()));
    }
    // Add user-specified parameter defaults
    arg_builder = arg_builder.with_defaults(parsed.param_defaults().clone());

    let r_formals = arg_builder.build_formals();
    let mut r_call_args_strs = arg_builder.build_call_args_vec();

    // Prepend .call parameter if using internal C wrapper
    if uses_internal_c_wrapper {
        r_call_args_strs.insert(0, ".call = match.call()".to_string());
    }

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
    // r_formals is already a joined string from build_formals()
    let formals_joined = r_formals;
    let roxygen_tags = crate::roxygen::roxygen_tags_from_attrs(attrs);
    let roxygen_tags_str = crate::roxygen::format_roxygen_tags(&roxygen_tags);
    let has_export_tag = crate::roxygen::has_roxygen_tag(&roxygen_tags, "export");
    // Add roxygen comments: @source for traceability, @export if public
    let source_comment = format!(
        "#' @source Generated by miniextendr from Rust fn `{}`\n",
        rust_ident
    );
    let export_comment = if matches!(vis, syn::Visibility::Public(_)) && !has_export_tag {
        "#' @export\n"
    } else {
        ""
    };
    let r_wrapper_string = format!(
        "{}{}{}{} <- function({}) {{\n    {}\n}}",
        roxygen_tags_str,
        source_comment,
        export_comment,
        r_wrapper_ident,
        formals_joined,
        r_wrapper_return_str
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
    let call_method_def_example = format!(
        "Value: `R_CallMethodDef {{ name: \"{}\", numArgs: {}, fun: <DL_FUNC> }}`",
        c_ident, num_args
    );

    // Get the normalized item for output, with roxygen tags stripped from docs.
    // Roxygen tags are for R documentation and shouldn't appear in rustdoc.
    let mut original_item = parsed.item_without_roxygen();
    // Strip only the miniextendr attributes; keep everything else.
    original_item
        .attrs
        .retain(|attr| !attr.path().is_ident("miniextendr"));
    let original_item = original_item;

    // Generate doc comment linking to C wrapper and R wrapper constant
    let fn_r_wrapper_doc = format!(
        "See [`{}`] for C wrapper, [`{}`] for R wrapper.",
        c_ident, r_wrapper_generator
    );

    let expanded: proc_macro::TokenStream = quote::quote! {
        // rust function with doc link to R wrapper
        #[doc = #fn_r_wrapper_doc]
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
        #[doc = #call_method_def_example]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        const #call_method_def: ::miniextendr_api::ffi::R_CallMethodDef = unsafe {
            ::miniextendr_api::ffi::R_CallMethodDef {
                name: #c_ident_name.as_ptr(),
                // Cast to DL_FUNC (generic function pointer) for storage in R's registration table.
                // R will cast back to the appropriate signature when calling.
                fun: Some(std::mem::transmute::<
                    unsafe #abi fn(#(#func_ptr_def),*) -> ::miniextendr_api::ffi::SEXP,
                    unsafe #abi fn() -> *mut ::std::os::raw::c_void
                >(#c_ident)),
                numArgs: #num_args,
            }
        };

        // doc-lint warnings (if any)
        #doc_lint_warnings
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
///     // Functions annotated with #[miniextendr]
///     fn my_function;
///     extern "C-unwind" fn C_my_raw_function;
///
///     // ALTREP types (registers the class with R)
///     struct MyAltrepClass;
///
///     // Impl blocks (class systems)
///     impl MyType;
///
///     // Trait impls (ABI dispatch)
///     impl Counter for MyType;
///
///     // Re-export from submodules
///     use submodule;
/// }
/// ```
///
/// # Function Registration
///
/// Functions listed here must be defined with the `#[miniextendr]` attribute.
/// The macro looks up the generated `CALL_METHOD_DEF_<name>` constant that
/// `#[miniextendr]` creates for each function.
///
/// The distinction between Rust ABI and C ABI functions is handled by
/// `#[miniextendr]` at the function definition site, not in this module declaration:
///
/// - **Rust ABI** (`fn foo(...)`): `#[miniextendr]` generates a `C_foo` wrapper
/// - **C ABI** (`extern "C-unwind" fn foo(...)`): `#[miniextendr]` uses the function directly,
///   and the R wrapper is prefixed with `unsafe_`
///
/// Both are listed the same way in `miniextendr_module!`:
///
/// ```ignore
/// miniextendr_module! {
///     mod mypackage;
///     fn rust_function;    // refers to #[miniextendr] fn rust_function
///     fn c_function;       // refers to #[miniextendr] extern "C-unwind" fn c_function
/// }
/// ```
///
/// # ALTREP Registration
///
/// Structs listed are registered as ALTREP classes during `R_init_*`.
/// The struct must implement the appropriate ALTREP traits.
///
/// # Impl and trait impl registration
///
/// - `impl Type;` registers a `#[miniextendr(...)] impl Type` block (env/S3/S4/S7/R6).
/// - `impl Trait for Type;` registers ABI wrappers for cross-package trait dispatch.
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
///     fn add;
///     extern "C-unwind" fn C_fast_add;
/// }
/// ```
#[proc_macro]
pub fn miniextendr_module(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_module = syn::parse_macro_input!(item as MiniextendrModule);

    let module = &parsed_module.module_name.ident;
    let module_entrypoint_ident = quote::format_ident!("R_init_{module}_miniextendr");
    // Build call entries with their cfg attributes preserved
    let call_entries_with_attrs: Vec<(Vec<syn::Attribute>, syn::Expr)> = parsed_module
        .functions
        .iter()
        .map(|f| {
            let call_method_def = f.call_method_def_ident();
            let cfg_attrs = extract_cfg_attrs(&f.attrs);
            (cfg_attrs, syn::parse_quote!(#call_method_def))
        })
        .collect();
    let call_entries: Vec<syn::Expr> = call_entries_with_attrs
        .iter()
        .map(|(_, expr)| expr.clone())
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

    // Generate impl block call defs for registration with cfg attributes
    let impl_call_defs_with_attrs: Vec<(Vec<syn::Attribute>, syn::Expr)> = parsed_module
        .impls
        .iter()
        .map(|i| {
            let call_defs_static = i.call_defs_const_ident();
            let cfg_attrs = extract_cfg_attrs(&i.attrs);
            (cfg_attrs, syn::parse_quote!(#call_defs_static))
        })
        .collect();
    let impl_call_defs: Vec<syn::Expr> = impl_call_defs_with_attrs
        .iter()
        .map(|(_, expr)| expr.clone())
        .collect();

    // Generate impl R wrapper refs
    let impl_r_wrappers: Vec<syn::Expr> = parsed_module
        .impls
        .iter()
        .map(|i| {
            let r_wrapper_const = i.r_wrappers_const_ident();
            syn::parse_quote!(#r_wrapper_const)
        })
        .collect();

    // Generate trait impl call defs for registration with cfg attributes
    let trait_impl_call_defs_with_attrs: Vec<(Vec<syn::Attribute>, syn::Expr)> = parsed_module
        .trait_impls
        .iter()
        .map(|ti| {
            let call_defs_static = ti.call_defs_const_ident();
            let cfg_attrs = extract_cfg_attrs(&ti.attrs);
            (cfg_attrs, syn::parse_quote!(#call_defs_static))
        })
        .collect();
    let trait_impl_call_defs: Vec<syn::Expr> = trait_impl_call_defs_with_attrs
        .iter()
        .map(|(_, expr)| expr.clone())
        .collect();

    // Generate trait impl R wrapper refs
    let trait_impl_r_wrappers: Vec<syn::Expr> = parsed_module
        .trait_impls
        .iter()
        .map(|ti| {
            let r_wrapper_const = ti.r_wrappers_const_ident();
            syn::parse_quote!(#r_wrapper_const)
        })
        .collect();

    // Get CALL_ENTRIES const arrays from child modules (via `use`)
    let use_module_call_entries_consts: Vec<syn::Expr> = parsed_module
        .uses
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let use_module_ident_upper = use_module_ident.to_string().to_uppercase();
            let call_entries_const = quote::format_ident!("CALL_ENTRIES_{use_module_ident_upper}");
            syn::parse_quote!(#use_module_ident::#call_entries_const)
        })
        .collect();

    // Call ALTREP registration from child modules (via `use`)
    let use_module_altrep_regs: Vec<syn::Expr> = parsed_module
        .uses
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let altrep_reg_fn = quote::format_ident!("{use_module_ident}_register_altrep");
            syn::parse_quote!(#use_module_ident::#altrep_reg_fn())
        })
        .collect();

    // region: r wrapper generation in `mod`

    let r_wrapper_generators: Vec<syn::Expr> = parsed_module
        .functions
        .iter()
        .map(|x| {
            let r_wrapper_const = x.r_wrapper_const_ident();
            syn::parse_quote!(#r_wrapper_const)
        })
        .collect();
    // Collect child modules' function wrappers (PARTS)
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

    // Collect child modules' impl wrappers (IMPLS)
    let r_wrappers_impl_use_other_modules = parsed_module
        .uses
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let use_module_ident_upper = use_module_ident.to_string().to_uppercase();
            let r_wrappers_use_module =
                quote::format_ident!("R_WRAPPERS_IMPLS_{use_module_ident_upper}");
            syn::parse_quote!(#use_module_ident::#r_wrappers_use_module)
        })
        .collect::<Vec<syn::Expr>>();

    let module_upper = module.to_string().to_uppercase();
    let r_wrappers_parts_ident = quote::format_ident!("R_WRAPPERS_PARTS_{module_upper}");
    let r_wrappers_deps_ident = quote::format_ident!("R_WRAPPERS_DEPS_{module_upper}");
    let r_wrappers_impl_deps_ident = quote::format_ident!("R_WRAPPERS_IMPL_DEPS_{module_upper}");

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
    let impl_links: Vec<String> = parsed_module
        .impls
        .iter()
        .map(|i| format!("[`{}`]", i.ident))
        .collect();
    let module_doc = if fn_links.is_empty() && struct_links.is_empty() && impl_links.is_empty() {
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
        if !impl_links.is_empty() {
            doc.push_str(&format!("- Impl blocks: {}\n", impl_links.join(", ")));
        }
        doc
    };

    // endregion

    // Check if we have impl blocks to register (affects wrapper lists)
    let _has_impls = !parsed_module.impls.is_empty();

    // Generate trait ABI wrapper infrastructure grouped by concrete type.
    let mut trait_impl_groups: Vec<(syn::Ident, Vec<syn::Path>)> = Vec::new();
    for ti in &parsed_module.trait_impls {
        if let Some((_, traits)) = trait_impl_groups
            .iter_mut()
            .find(|(ty, _)| ty == &ti.type_ident)
        {
            traits.push(ti.trait_path.clone());
        } else {
            trait_impl_groups.push((ti.type_ident.clone(), vec![ti.trait_path.clone()]));
        }
    }
    let trait_impl_wrappers: Vec<proc_macro2::TokenStream> = trait_impl_groups
        .iter()
        .map(|(type_ident, trait_paths)| generate_trait_impl_wrapper(trait_paths, type_ident))
        .collect();

    // R wrapper parts const (includes both functions and impl wrappers)
    let r_wrappers_impls_ident = quote::format_ident!("R_WRAPPERS_IMPLS_{module_upper}");

    // Generate CALL_ENTRIES constant name
    let call_entries_const_ident = quote::format_ident!("CALL_ENTRIES_{module_upper}");

    // Generate call_entries accessor function name (returns &[R_CallMethodDef])
    let call_entries_fn_ident = quote::format_ident!("{module}_call_entries");

    // Generate ALTREP registration function name
    let altrep_reg_fn_ident = quote::format_ident!("{module}_register_altrep");

    // Build const call entries array, including impl call defs and trait impl call defs.
    let call_entries_len_lit = syn::LitInt::new(
        &call_entries_len.to_string(),
        proc_macro2::Span::call_site(),
    );
    let impl_call_defs_len_exprs: Vec<proc_macro2::TokenStream> = parsed_module
        .impls
        .iter()
        .map(|i| {
            let call_defs_static = i.call_defs_const_ident();
            quote::quote!(#call_defs_static.len())
        })
        .collect();
    let trait_impl_call_defs_len_exprs: Vec<proc_macro2::TokenStream> = parsed_module
        .trait_impls
        .iter()
        .map(|ti| {
            let call_defs_static = ti.call_defs_const_ident();
            quote::quote!(#call_defs_static.len())
        })
        .collect();

    // Calculate total length expression
    let all_len_exprs: Vec<proc_macro2::TokenStream> =
        std::iter::once(quote::quote!(#call_entries_len_lit))
            .chain(impl_call_defs_len_exprs.iter().cloned())
            .chain(trait_impl_call_defs_len_exprs.iter().cloned())
            .collect();
    let total_len_expr = if all_len_exprs.is_empty()
        || (call_entries_len == 0
            && impl_call_defs_len_exprs.is_empty()
            && trait_impl_call_defs_len_exprs.is_empty())
    {
        quote::quote!(0usize)
    } else {
        quote::quote!(#(#all_len_exprs)+*)
    };

    // Generate conditional call entry assignment statements
    let call_entry_assignments: Vec<proc_macro2::TokenStream> = call_entries_with_attrs
        .iter()
        .map(|(cfg_attrs, expr)| {
            quote::quote! {
                #(#cfg_attrs)*
                {
                    entries[idx] = #expr;
                    idx += 1;
                }
            }
        })
        .collect();

    let call_entries_storage = quote::quote! {
        /// This module's call entries (excluding children).
        #[doc(hidden)]
        pub(crate) const #call_entries_const_ident: [::miniextendr_api::ffi::R_CallMethodDef; #total_len_expr] = {
            const EMPTY: ::miniextendr_api::ffi::R_CallMethodDef = ::miniextendr_api::ffi::R_CallMethodDef {
                name: std::ptr::null(),
                fun: None,
                numArgs: 0,
            };
            let mut entries = [EMPTY; #total_len_expr];
            let mut idx: usize = 0;
            #(#call_entry_assignments)*
            #(
                let mut j: usize = 0;
                let slice = &#impl_call_defs;
                while j < slice.len() {
                    entries[idx] = slice[j];
                    idx += 1;
                    j += 1;
                }
            )*
            #(
                let mut j: usize = 0;
                let slice = &#trait_impl_call_defs;
                while j < slice.len() {
                    entries[idx] = slice[j];
                    idx += 1;
                    j += 1;
                }
            )*
            entries
        };

        /// Returns this module's call entries as a slice.
        fn #call_entries_fn_ident() -> &'static [::miniextendr_api::ffi::R_CallMethodDef] {
            &#call_entries_const_ident
        }
    };

    // Build a combined const array including child modules and a sentinel.
    let use_module_call_entries_len_exprs: Vec<proc_macro2::TokenStream> =
        use_module_call_entries_consts
            .iter()
            .map(|expr| quote::quote!(#expr.len()))
            .collect();
    let all_call_entries_const_ident = quote::format_ident!("ALL_CALL_ENTRIES_{module_upper}");
    let all_entries_len_expr = if use_module_call_entries_len_exprs.is_empty() {
        quote::quote!(#total_len_expr + 1usize)
    } else {
        quote::quote!(#total_len_expr + #(#use_module_call_entries_len_exprs)+* + 1usize)
    };
    let all_call_entries_storage = quote::quote! {
        /// This module's call entries including children, with sentinel.
        #[doc(hidden)]
        const #all_call_entries_const_ident: [::miniextendr_api::ffi::R_CallMethodDef; #all_entries_len_expr] = {
            const EMPTY: ::miniextendr_api::ffi::R_CallMethodDef = ::miniextendr_api::ffi::R_CallMethodDef {
                name: std::ptr::null(),
                fun: None,
                numArgs: 0,
            };
            let mut entries = [EMPTY; #all_entries_len_expr];
            let mut idx: usize = 0;

            // Local entries
            let mut j: usize = 0;
            let slice = &#call_entries_const_ident;
            while j < slice.len() {
                entries[idx] = slice[j];
                idx += 1;
                j += 1;
            }

            // Child module entries
            #(
                let mut j: usize = 0;
                let slice = &#use_module_call_entries_consts;
                while j < slice.len() {
                    entries[idx] = slice[j];
                    idx += 1;
                    j += 1;
                }
            )*

            // Sentinel
            entries[idx] = ::miniextendr_api::ffi::R_CallMethodDef {
                name: std::ptr::null(),
                fun: None,
                numArgs: 0,
            };

            entries
        };
    };

    // Check if we have trait impl blocks to register
    let _has_trait_impls = !parsed_module.trait_impls.is_empty();

    // Generate R wrapper impls constant - includes both impl and trait impl wrappers
    let mut all_impl_r_wrappers = Vec::new();
    all_impl_r_wrappers.extend(impl_r_wrappers.clone());
    all_impl_r_wrappers.extend(trait_impl_r_wrappers.clone());

    let r_wrappers_impls_const = if all_impl_r_wrappers.is_empty() {
        quote::quote! {
            pub const #r_wrappers_impls_ident: &[&str] = &[];
        }
    } else {
        quote::quote! {
            pub const #r_wrappers_impls_ident: &[&str] = &[#(#all_impl_r_wrappers),*];
        }
    };

    // Generate the module - common structure for both cases
    quote::quote! {
        #[doc(hidden)]
        pub const #r_wrappers_parts_ident: &[&str] = &[#(#r_wrapper_generators),*];
        #[doc(hidden)]
        #r_wrappers_impls_const
        #[doc(hidden)]
        pub const #r_wrappers_deps_ident: &[&[&str]] = &[#(#r_wrappers_use_other_modules),*];
        #[doc(hidden)]
        pub const #r_wrappers_impl_deps_ident: &[&[&str]] = &[#(#r_wrappers_impl_use_other_modules),*];

        #call_entries_storage
        #all_call_entries_storage

        // Trait ABI wrapper infrastructure
        #(#trait_impl_wrappers)*

        /// Register ALTREP classes declared in this module.
        pub(crate) fn #altrep_reg_fn_ident() {
            #(#altrep_regs;)*
        }

        #[doc = #module_doc]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        extern "C-unwind" fn #module_entrypoint_ident(dll: *mut ::miniextendr_api::ffi::DllInfo) {
            // Register ALTREP classes from this module
            #altrep_reg_fn_ident();

            // Register ALTREP classes from child modules
            #(#use_module_altrep_regs;)*

            unsafe {
                ::miniextendr_api::ffi::R_registerRoutines_unchecked(
                    dll,
                    std::ptr::null(),
                    #all_call_entries_const_ident.as_ptr(),
                    std::ptr::null(),
                    std::ptr::null()
                );
                // R_useDynamicSymbols and R_forceSymbols are called in entrypoint.c
            }
        }
    }
    .into()
}

/// Generate wrapper infrastructure for a trait implementation.
///
/// For `impl Counter for SimpleCounter;` generates:
/// - `__MxWrapperSimpleCounter` - Type-erased wrapper struct
/// - `__MX_BASE_VTABLE_SIMPLECOUNTER` - Base vtable
/// - `__mx_wrap_simplecounter()` - Constructor
///
/// If multiple `impl Trait for Type;` entries exist for the same concrete type,
/// the generated query function includes all listed traits.
fn generate_trait_impl_wrapper(
    trait_paths: &[syn::Path],
    type_ident: &syn::Ident,
) -> proc_macro2::TokenStream {
    let type_upper = type_ident.to_string().to_uppercase();
    let type_lower = type_ident.to_string().to_lowercase();

    // Generate identifiers
    let wrapper_name = quote::format_ident!("__MxWrapper{}", type_ident);
    let base_vtable_name = quote::format_ident!("__MX_BASE_VTABLE_{}", type_upper);
    let concrete_tag_name = quote::format_ident!("__MX_TAG_{}", type_upper);
    let drop_fn_name = quote::format_ident!("__mx_drop_{}", type_lower);
    let query_fn_name = quote::format_ident!("__mx_query_{}", type_lower);
    let wrap_fn_name = quote::format_ident!("__mx_wrap_{}", type_lower);

    // Generate tag path string for hashing
    let tag_path = format!("::{}", type_ident);

    // Generate query branches for each trait
    let query_branches: Vec<proc_macro2::TokenStream> = trait_paths
        .iter()
        .map(|trait_path| {
            let trait_name = trait_path
                .segments
                .last()
                .map(|s| &s.ident)
                .expect("trait path has at least one segment");
            let trait_name_upper = trait_name.to_string().to_uppercase();

            // Build TAG path - same module as trait
            let mut trait_tag_path = trait_path.clone();
            if let Some(last) = trait_tag_path.segments.last_mut() {
                last.ident = quote::format_ident!("TAG_{}", trait_name_upper);
            }

            // Build vtable static name: __VTABLE_{TRAIT}_FOR_{TYPE}
            let vtable_name =
                quote::format_ident!("__VTABLE_{}_FOR_{}", trait_name_upper, type_upper);

            quote::quote! {
                if trait_tag == #trait_tag_path {
                    return &#vtable_name as *const _ as *const ::std::os::raw::c_void;
                }
            }
        })
        .collect();

    quote::quote! {
        #[doc = concat!(
            "Type-erased wrapper for `",
            stringify!(#type_ident),
            "` with trait dispatch support."
        )]
        #[doc = "Generated by `miniextendr_module!`."]
        #[repr(C)]
        #[doc(hidden)]
        struct #wrapper_name {
            /// Type-erased header. Must be first field for `mx_erased` compatibility.
            pub erased: ::miniextendr_api::abi::mx_erased,
            /// The actual data.
            pub data: #type_ident,
        }

        #[doc = concat!(
            "Concrete type tag for `",
            stringify!(#type_ident),
            "`."
        )]
        #[doc(hidden)]
        const #concrete_tag_name: ::miniextendr_api::abi::mx_tag =
            ::miniextendr_api::abi::mx_tag_from_path(concat!(module_path!(), #tag_path));

        #[doc = concat!(
            "Drop function for `",
            stringify!(#type_ident),
            "` wrapper."
        )]
        #[doc(hidden)]
        unsafe extern "C" fn #drop_fn_name(ptr: *mut ::miniextendr_api::abi::mx_erased) {
            if ptr.is_null() {
                return;
            }
            let wrapper = ptr as *mut #wrapper_name;
            unsafe { drop(Box::from_raw(wrapper)); }
        }

        #[doc = concat!(
            "Query function for `",
            stringify!(#type_ident),
            "` trait dispatch."
        )]
        #[doc(hidden)]
        unsafe extern "C" fn #query_fn_name(
            _ptr: *mut ::miniextendr_api::abi::mx_erased,
            trait_tag: ::miniextendr_api::abi::mx_tag,
        ) -> *const ::std::os::raw::c_void {
            #(#query_branches)*
            ::std::ptr::null()
        }

        #[doc = concat!(
            "Base vtable for `",
            stringify!(#type_ident),
            "`."
        )]
        #[doc(hidden)]
        static #base_vtable_name: ::miniextendr_api::abi::mx_base_vtable =
            ::miniextendr_api::abi::mx_base_vtable {
                drop: #drop_fn_name,
                concrete_tag: #concrete_tag_name,
                query: #query_fn_name,
            };

        #[doc = concat!(
            "Create a type-erased wrapper for `",
            stringify!(#type_ident),
            "`."
        )]
        #[doc(hidden)]
        fn #wrap_fn_name(data: #type_ident) -> *mut ::miniextendr_api::abi::mx_erased {
            let wrapper = Box::new(#wrapper_name {
                erased: ::miniextendr_api::abi::mx_erased {
                    base: &#base_vtable_name,
                },
                data,
            });
            Box::into_raw(wrapper) as *mut ::miniextendr_api::abi::mx_erased
        }
    }
}

/// Generate thread-safe wrappers for R FFI functions.
///
/// Apply this to an `extern "C-unwind"` block to generate wrappers that ensure
/// R API calls happen on R's main thread.
///
/// # Behavior by Return Type
///
/// The wrapper behavior depends on the return type:
///
/// ## Value-returning functions
/// Functions returning values (SEXP, i32, etc.) are automatically routed to
/// the main thread via `with_r_thread` when called from a worker thread.
///
/// ## Pointer-returning functions
/// Functions returning raw pointers (`*const T`, `*mut T`) **cannot be routed**
/// and will **panic** if called from a non-main thread. This is because the
/// pointer could become invalid when R's garbage collector runs on the main thread.
///
/// For pointer-returning APIs (like `INTEGER`, `REAL`), you must:
/// - Call them from the main thread, OR
/// - Use `with_r_thread(|| { ... })` to execute pointer operations on main thread
///   and process results before returning
///
/// # Initialization Requirement
///
/// `miniextendr_worker_init()` must be called before using any wrapped function.
/// Calling before initialization will panic with a descriptive error message.
///
/// # Limitations
///
/// - Variadic functions are passed through unchanged (no wrapper)
/// - Statics are passed through unchanged
/// - Functions with `#[link_name]` are passed through unchanged
///
/// # Example
///
/// ```ignore
/// #[r_ffi_checked]
/// unsafe extern "C-unwind" {
///     // Value-returning: automatically routed from worker threads
///     pub fn Rf_ScalarInteger(arg1: i32) -> SEXP;
///
///     // Pointer-returning: panics if called from worker thread
///     pub fn INTEGER(x: SEXP) -> *mut i32;
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

                    let is_never = matches!(output, syn::ReturnType::Type(_, ty) if matches!(**ty, syn::Type::Never(_)));

                    // Check if return type is a raw pointer (*const T or *mut T)
                    // These MUST NOT be routed - the pointer would be invalid on the worker thread
                    let returns_raw_pointer = matches!(output, syn::ReturnType::Type(_, ty) if matches!(**ty, syn::Type::Ptr(_)));

                    let wrapper = if is_never {
                        // Never-returning functions (like Rf_error)
                        quote::quote! {
                            #(#attrs)*
                            #[inline(always)]
                            #[allow(non_snake_case)]
                            #vis unsafe fn #fn_name(#inputs) #output {
                                ::miniextendr_api::worker::with_r_thread(move || unsafe {
                                    #unchecked_name(#(#arg_names),*)
                                })
                            }
                        }
                    } else if returns_raw_pointer {
                        // Pointer-returning functions MUST be called on main thread.
                        // Routing would return a pointer that could become invalid when
                        // R's GC runs on the main thread.
                        let fn_name_str = fn_name.to_string();
                        quote::quote! {
                            #(#attrs)*
                            #[inline(always)]
                            #[allow(non_snake_case)]
                            #vis unsafe fn #fn_name(#inputs) #output {
                                ::miniextendr_api::worker::assert_r_main_thread_for_pointer_api(#fn_name_str);
                                #unchecked_name(#(#arg_names),*)
                            }
                        }
                    } else {
                        // Normal functions - route via with_r_thread
                        quote::quote! {
                            #(#attrs)*
                            #[inline(always)]
                            #[allow(non_snake_case)]
                            #vis unsafe fn #fn_name(#inputs) #output {
                                let result = ::miniextendr_api::worker::with_r_thread(move || {
                                    ::miniextendr_api::worker::Sendable(unsafe {
                                        #unchecked_name(#(#arg_names),*)
                                    })
                                });
                                result.0
                            }
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
///
///     unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
///         <i32 as RNativeType>::dataptr_mut(sexp).cast()
///     }
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

            #[inline]
            unsafe fn dataptr_mut(sexp: ::miniextendr_api::ffi::SEXP) -> *mut Self {
                // Newtype is repr(transparent), so we can cast the pointer
                unsafe {
                    <#inner_ty as ::miniextendr_api::ffi::RNativeType>::dataptr_mut(sexp).cast()
                }
            }
        }

    };

    expanded.into()
}

/// Derive macro for implementing `TypedExternal` on a type.
///
/// This makes the type compatible with `ExternalPtr<T>` for storing in R's external pointers.
///
/// # Basic Usage
///
/// ```ignore
/// use miniextendr_api::TypedExternal;
///
/// #[derive(ExternalPtr)]
/// struct MyData {
///     value: i32,
/// }
///
/// // Now you can use ExternalPtr<MyData>
/// let ptr = ExternalPtr::new(MyData { value: 42 });
/// ```
///
/// # Trait ABI
///
/// To enable trait dispatch wrappers, list trait impls in `miniextendr_module!`:
///
/// ```ignore
/// use miniextendr_api::{miniextendr, miniextendr_module};
///
/// #[derive(ExternalPtr)]
/// struct MyCounter {
///     value: i32,
/// }
///
/// #[miniextendr]
/// impl Counter for MyCounter {
///     fn value(&self) -> i32 { self.value }
///     fn increment(&mut self) { self.value += 1; }
/// }
///
/// miniextendr_module! {
///     mod mypkg;
///     impl Counter for MyCounter;
/// }
/// ```
///
/// This generates additional infrastructure for type-erased trait dispatch:
/// - `__MxWrapperMyCounter` - Type-erased wrapper struct
/// - `__MX_BASE_VTABLE_MYCOUNTER` - Base vtable with drop/query
/// - `__mx_wrap_mycounter()` - Constructor returning `*mut mx_erased`
///
/// # Generated Code (Basic)
///
/// For a type `MyData` without traits:
///
/// ```ignore
/// impl TypedExternal for MyData {
///     const TYPE_NAME: &'static str = "MyData";
///     const TYPE_NAME_CSTR: &'static [u8] = b"MyData\0";
/// }
/// ```
#[proc_macro_derive(ExternalPtr, attributes(externalptr))]
pub fn derive_external_ptr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    externalptr_derive::derive_external_ptr(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP integer vector data types.
///
/// Auto-implements `AltrepLen`, `AltIntegerData`, and calls `impl_altinteger_from_data!`.
///
/// # Attributes
///
/// - `#[altrep(len = "field_name")]` - Specify length field (auto-detects "len" or "length")
/// - `#[altrep(elt = "field_name")]` - For constant vectors, specify which field provides elements
/// - `#[altrep(dataptr)]` - Pass `dataptr` option to low-level macro
/// - `#[altrep(serialize)]` - Pass `serialize` option to low-level macro
/// - `#[altrep(subset)]` - Pass `subset` option to low-level macro
/// - `#[altrep(no_lowlevel)]` - Skip automatic `impl_altinteger_from_data!` call
///
/// # Example (Constant Vector - Zero Boilerplate!)
///
/// ```ignore
/// #[derive(ExternalPtr, AltrepInteger)]
/// #[altrep(elt = "value")]  // All elements return this field
/// pub struct ConstantIntData {
///     value: i32,
///     len: usize,
/// }
///
/// // That's it! 3 lines instead of 30!
/// // AltrepLen, AltIntegerData, and low-level impls are auto-generated
///
/// #[miniextendr(class = "ConstantInt", pkg = "mypkg")]
/// pub struct ConstantIntClass(pub ConstantIntData);
/// ```
///
/// # Example (Custom elt() - Override One Method)
///
/// ```ignore
/// #[derive(ExternalPtr, AltrepInteger)]
/// pub struct ArithSeqData {
///     start: i32,
///     step: i32,
///     len: usize,
/// }
///
/// // Auto-generates AltrepLen and stub AltIntegerData
/// // Just override elt() for custom logic:
/// impl AltIntegerData for ArithSeqData {
///     fn elt(&self, i: usize) -> i32 {
///         self.start + (i as i32) * self.step
///     }
/// }
/// ```
#[proc_macro_derive(AltrepInteger, attributes(altrep))]
pub fn derive_altrep_integer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_integer(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP real vector data types.
///
/// Auto-implements `AltrepLen` and `AltRealData` traits.
/// Supports the same `#[altrep(...)]` attributes as `AltrepInteger`.
#[proc_macro_derive(AltrepReal, attributes(altrep))]
pub fn derive_altrep_real(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_real(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP logical vector data types.
///
/// Auto-implements `AltrepLen` and `AltLogicalData` traits.
/// Supports the same `#[altrep(...)]` attributes as `AltrepInteger`.
#[proc_macro_derive(AltrepLogical, attributes(altrep))]
pub fn derive_altrep_logical(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_logical(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP raw vector data types.
///
/// Auto-implements `AltrepLen` and `AltRawData` traits.
/// Supports the same `#[altrep(...)]` attributes as `AltrepInteger`.
#[proc_macro_derive(AltrepRaw, attributes(altrep))]
pub fn derive_altrep_raw(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_raw(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP string vector data types.
///
/// Auto-implements `AltrepLen` and `AltStringData` traits.
/// Supports the same `#[altrep(...)]` attributes as `AltrepInteger`.
#[proc_macro_derive(AltrepString, attributes(altrep))]
pub fn derive_altrep_string(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_string(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP complex vector data types.
///
/// Auto-implements `AltrepLen` and `AltComplexData` traits.
/// Supports the same `#[altrep(...)]` attributes as `AltrepInteger`.
#[proc_macro_derive(AltrepComplex, attributes(altrep))]
pub fn derive_altrep_complex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_complex(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive macro for ALTREP list vector data types.
///
/// Auto-implements `AltrepLen` and `AltListData` traits.
/// Supports the same `#[altrep(...)]` attributes as `AltrepInteger`.
#[proc_macro_derive(AltrepList, attributes(altrep))]
pub fn derive_altrep_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    altrep_derive::derive_altrep_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `IntoList` for a struct (Rust → R list).
///
/// - Named structs → named R list: `list(x = 1L, y = 2L)`
/// - Tuple structs → unnamed R list: `list(1L, 2L)`
/// - Fields annotated `#[into_list(ignore)]` are skipped
#[proc_macro_derive(IntoList, attributes(into_list))]
pub fn derive_into_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_into_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `TryFromList` for a struct (R list → Rust).
///
/// - Named structs: extract by field name
/// - Tuple structs: extract by position (0, 1, 2, ...)
/// - Fields annotated `#[into_list(ignore)]` are not read and are initialized with `Default::default()`
#[proc_macro_derive(TryFromList, attributes(into_list))]
pub fn derive_try_from_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_try_from_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PrefersList`: opt into list-first IntoR by implementing the marker and IntoR wrapper.
#[proc_macro_derive(PreferList)]
pub fn derive_prefer_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_list(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferExternalPtr`: marks a type as preferring ExternalPtr conversion.
#[proc_macro_derive(PreferExternalPtr)]
pub fn derive_prefer_externalptr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_externalptr(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derive `PreferRNativeType`: marks a type as preferring native SEXP conversion.
#[proc_macro_derive(PreferRNativeType)]
pub fn derive_prefer_rnative(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    list_derive::derive_prefer_rnative(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

#[cfg(test)]
mod tests;
