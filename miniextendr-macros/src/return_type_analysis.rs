//! Return type analysis for `#[miniextendr]` functions.
//!
//! Determines:
//! 1. Whether function returns SEXP (affects thread strategy)
//! 2. Whether result should be invisible in R
//! 3. How to convert Rust return value to SEXP
//! 4. What post-call processing is needed (unwrapping Option/Result)

use crate::is_sexp_type;
use syn::spanned::Spanned;

/// Mutable context threaded through `analyze_option_type` and `analyze_result_type`.
///
/// Collects analysis results that are determined while recursively inspecting
/// the inner types of `Option<T>` and `Result<T, E>`.
struct AnalysisCtx<'a> {
    /// Identifier for the variable holding the Rust function's return value.
    rust_result_ident: &'a syn::Ident,
    /// Set to `true` if the return type contains SEXP, meaning the function must
    /// run on R's main thread (SEXP is not `Send`).
    returns_sexp: &'a mut bool,
    /// Set to `true` if the R wrapper should return invisibly (e.g., `()` or `Option<()>`).
    is_invisible: &'a mut bool,
    /// Statements to execute after calling the Rust function but before converting
    /// the result to SEXP (e.g., unwrapping `Result::Err` to an R error).
    post_call_statements: &'a mut Vec<proc_macro2::TokenStream>,
}

/// Complete analysis result for a `#[miniextendr]` function's return type.
///
/// Captures everything needed to generate the C wrapper's return handling:
/// which thread strategy to use, whether the R wrapper returns invisibly,
/// how to convert the Rust value to SEXP, and any intermediate unwrapping steps.
pub(crate) struct ReturnTypeAnalysis {
    /// Whether the return type contains SEXP (affects thread strategy).
    ///
    /// When `true`, the generated wrapper runs on R's main thread because
    /// SEXP is not `Send`.
    pub returns_sexp: bool,

    /// Whether the R wrapper function should return its result invisibly.
    ///
    /// Set to `true` for `()`, `Option<()>`, and `Result<(), E>` return types.
    pub is_invisible: bool,

    /// TokenStream that converts the Rust result variable to a SEXP value.
    ///
    /// This expression references `rust_result_ident` and produces the final
    /// SEXP to return to R, using `IntoR::into_sexp`, strict checked conversion,
    /// or direct passthrough depending on the type.
    pub return_expression: proc_macro2::TokenStream,

    /// Statements inserted between the Rust function call and the return expression.
    ///
    /// Used for unwrapping `Result::Err` (raises R error) or checking `Option::None`
    /// before the conversion to SEXP occurs.
    pub post_call_statements: Vec<proc_macro2::TokenStream>,
}

/// Analyze a function's return type and generate conversion code.
///
/// This is the main entry point for return type analysis. It pattern-matches on the
/// return type to determine how to convert the Rust result into a SEXP for R.
///
/// # Parameters
/// - `output`: The function's return type from `syn::Signature`
/// - `rust_result_ident`: Identifier for the variable holding the Rust function result
/// - `rust_ident`: Function name (used in error messages for `Option::None`)
/// - `return_pref`: How to convert non-primitive return types (Auto, List, ExternalPtr, Native)
/// - `unwrap_in_r`: When `true`, `Result<T, E>` is passed to R as-is via `IntoR` (list with error field)
///   rather than unwrapped in Rust (which raises an R error)
/// - `strict`: When `true`, lossy integer types (i64, u64, isize, usize) use checked
///   conversions that panic on overflow instead of silent truncation
/// - `error_in_r`: When `true`, errors are returned as tagged SEXP values (checked in R)
///   instead of raising an R error directly in Rust
pub(crate) fn analyze_return_type(
    output: &syn::ReturnType,
    rust_result_ident: &syn::Ident,
    rust_ident: &syn::Ident,
    return_pref: crate::miniextendr_fn::ReturnPref,
    unwrap_in_r: bool,
    strict: bool,
    error_in_r: bool,
) -> ReturnTypeAnalysis {
    let mut returns_sexp = false;
    let mut is_invisible = false;
    let mut post_call_statements = Vec::new();

    let fn_name_str = rust_ident.to_string();
    let option_none_error_msg = format!("miniextendr function `{fn_name_str}` returned None");

    let return_expression = match output {
        // No return type (no arrow)
        syn::ReturnType::Default => {
            is_invisible = true;
            quote::quote! { ::miniextendr_api::ffi::SEXP::null() }
        }

        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            // -> ()
            syn::Type::Tuple(t) if t.elems.is_empty() => {
                is_invisible = true;
                quote::quote! { ::miniextendr_api::ffi::SEXP::null() }
            }

            // -> SEXP
            syn::Type::Path(_p) if is_sexp_type(ty.as_ref()) => {
                is_invisible = false;
                returns_sexp = true;
                quote::quote! { #rust_result_ident }
            }

            // -> Option<T>
            syn::Type::Path(p)
                if p.path.segments.last().map(|s| &s.ident)
                    == Some(&syn::Ident::new("Option", p.path.span())) =>
            {
                let mut ctx = AnalysisCtx {
                    rust_result_ident,
                    returns_sexp: &mut returns_sexp,
                    is_invisible: &mut is_invisible,
                    post_call_statements: &mut post_call_statements,
                };
                analyze_option_type(p, &mut ctx, &option_none_error_msg, strict, error_in_r)
            }

            // -> Result<T, E>
            syn::Type::Path(p)
                if p.path.segments.last().map(|s| &s.ident)
                    == Some(&syn::Ident::new("Result", p.path.span())) =>
            {
                let mut ctx = AnalysisCtx {
                    rust_result_ident,
                    returns_sexp: &mut returns_sexp,
                    is_invisible: &mut is_invisible,
                    post_call_statements: &mut post_call_statements,
                };
                analyze_result_type(p, &mut ctx, unwrap_in_r, error_in_r)
            }

            // -> T (any other type)
            _ => {
                is_invisible = false;
                // When strict mode is enabled and return type is lossy, use checked conversion
                if strict
                    && let Some(strict_expr) =
                        strict_conversion_for_type(ty.as_ref(), rust_result_ident)
                {
                    return ReturnTypeAnalysis {
                        returns_sexp,
                        is_invisible,
                        return_expression: strict_expr,
                        post_call_statements,
                    };
                }
                match return_pref {
                    crate::miniextendr_fn::ReturnPref::List => {
                        quote::quote! {
                            ::miniextendr_api::into_r::IntoR::into_sexp(
                                ::miniextendr_api::convert::AsList(#rust_result_ident)
                            )
                        }
                    }
                    crate::miniextendr_fn::ReturnPref::ExternalPtr => {
                        quote::quote! {
                            ::miniextendr_api::into_r::IntoR::into_sexp(
                                ::miniextendr_api::convert::AsExternalPtr(#rust_result_ident)
                            )
                        }
                    }
                    crate::miniextendr_fn::ReturnPref::Native => {
                        quote::quote! {
                            ::miniextendr_api::into_r::IntoR::into_sexp(
                                ::miniextendr_api::convert::AsRNative(#rust_result_ident)
                            )
                        }
                    }
                    crate::miniextendr_fn::ReturnPref::Auto => {
                        quote::quote! {
                            ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident)
                        }
                    }
                }
            }
        },
    };

    ReturnTypeAnalysis {
        returns_sexp,
        is_invisible,
        return_expression,
        post_call_statements,
    }
}

/// Analyze `Option<T>` return type and generate conversion code.
///
/// Handles three cases:
/// - `Option<()>`: invisible, errors on `None`
/// - `Option<SEXP>`: returns the SEXP or `R_NilValue` for `None`
/// - `Option<T>`: delegates to `IntoR` (which maps `None` to `NA` for supported types)
///
/// In `error_in_r` mode, `None` returns a tagged error SEXP instead of raising an R error.
fn analyze_option_type(
    type_path: &syn::TypePath,
    ctx: &mut AnalysisCtx,
    option_none_error_msg: &str,
    strict: bool,
    error_in_r: bool,
) -> proc_macro2::TokenStream {
    let rust_result_ident = ctx.rust_result_ident;
    let seg = type_path.path.segments.last().unwrap();
    let inner_ty = crate::first_type_argument(seg);
    let is_unit_inner =
        inner_ty.is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));
    let is_sexp_inner = inner_ty.is_some_and(is_sexp_type);

    if is_unit_inner {
        // Option<()> - invisible, error on None
        *ctx.is_invisible = true;
        if error_in_r {
            // error_in_r: return tagged error value on None (no post_call unwrap)
            quote::quote! {
                match #rust_result_ident {
                    Some(()) => ::miniextendr_api::ffi::SEXP::null(),
                    None => ::miniextendr_api::error_value::make_rust_error_value(
                        #option_none_error_msg, "none_err", Some(__miniextendr_call),
                    ),
                }
            }
        } else {
            ctx.post_call_statements.push(quote::quote! {
                if #rust_result_ident.is_none() {
                    ::miniextendr_api::error::r_stop(#option_none_error_msg);
                }
            });
            quote::quote! { ::miniextendr_api::ffi::SEXP::null() }
        }
    } else if is_sexp_inner {
        // Option<SEXP> - return SEXP or R_NilValue for None
        *ctx.is_invisible = false;
        *ctx.returns_sexp = true;
        quote::quote! {
            match #rust_result_ident {
                Some(v) => v,
                None => ::miniextendr_api::ffi::SEXP::null(),
            }
        }
    } else {
        // Option<T> - convert via IntoR which handles None → NA appropriately
        *ctx.is_invisible = false;
        // In strict mode, check if this is Option<lossy> and use checked conversion
        if strict
            && let Some(strict_expr) =
                strict_conversion_for_type(&syn::Type::Path(type_path.clone()), rust_result_ident)
        {
            return strict_expr;
        }
        quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
    }
}

/// Analyze `Result<T, E>` return type and generate conversion code.
///
/// Handles several combinations of `T` and `E`:
/// - `Result<T, ()>`: unit error is a deliberate sentinel; `Err(())` returns `NULL` to R
/// - `Result<(), E>`: invisible, `Err` raises R error (or tagged error in `error_in_r` mode)
/// - `Result<SEXP, E>`: returns the SEXP directly on `Ok`
/// - `Result<T, E>` with `unwrap_in_r`: passes the full `Result` to R as a list
/// - `Result<T, E>` default: unwraps in Rust, raising R error on `Err`
fn analyze_result_type(
    type_path: &syn::TypePath,
    ctx: &mut AnalysisCtx,
    unwrap_in_r: bool,
    error_in_r: bool,
) -> proc_macro2::TokenStream {
    let rust_result_ident = ctx.rust_result_ident;
    let seg = type_path.path.segments.last().unwrap();
    let ok_ty = crate::first_type_argument(seg);
    let err_ty = crate::second_type_argument(seg);
    let ok_is_unit =
        ok_ty.is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));
    let ok_is_sexp = ok_ty.is_some_and(is_sexp_type);
    let err_is_unit =
        err_ty.is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));

    // Special case: Result<T, ()> - convert to Result<T, NullOnErr> which returns NULL on Err
    // Unit error is a deliberate sentinel, not a Rust failure - error_in_r does not change this.
    if err_is_unit {
        if ok_is_unit {
            // Result<(), ()> - invisible, always returns NULL
            *ctx.is_invisible = true;
            quote::quote! { ::miniextendr_api::ffi::SEXP::null() }
        } else {
            // Result<T, ()> - convert to Result<T, NullOnErr> and use IntoR
            // IntoR for Result<T, NullOnErr> returns NULL on Err
            *ctx.is_invisible = false;
            if ok_is_sexp {
                *ctx.returns_sexp = true;
            }
            // Convert Err(()) to Err(NullOnErr) so IntoR can return NULL
            ctx.post_call_statements.push(quote::quote! {
                let #rust_result_ident = #rust_result_ident.map_err(|()| ::miniextendr_api::into_r::NullOnErr);
            });
            // Use IntoR which returns NULL on Err(NullOnErr)
            quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
        }
    } else if error_in_r {
        // error_in_r mode: return tagged error value on Err (takes priority over unwrap_in_r)
        if ok_is_unit {
            // Result<(), E> - invisible, return error value on Err
            *ctx.is_invisible = true;
            quote::quote! {
                match #rust_result_ident {
                    Ok(()) => ::miniextendr_api::ffi::SEXP::null(),
                    Err(e) => ::miniextendr_api::error_value::make_rust_error_value(
                        &format!("{:?}", e), "result_err", Some(__miniextendr_call),
                    ),
                }
            }
        } else if ok_is_sexp {
            // Result<SEXP, E> - return SEXP or error value
            *ctx.is_invisible = false;
            *ctx.returns_sexp = true;
            quote::quote! {
                match #rust_result_ident {
                    Ok(v) => v,
                    Err(e) => ::miniextendr_api::error_value::make_rust_error_value(
                        &format!("{:?}", e), "result_err", Some(__miniextendr_call),
                    ),
                }
            }
        } else {
            // Result<T, E> - convert Ok to SEXP, return error value on Err
            *ctx.is_invisible = false;
            quote::quote! {
                match #rust_result_ident {
                    Ok(v) => ::miniextendr_api::into_r::IntoR::into_sexp(v),
                    Err(e) => ::miniextendr_api::error_value::make_rust_error_value(
                        &format!("{:?}", e), "result_err", Some(__miniextendr_call),
                    ),
                }
            }
        }
    } else if unwrap_in_r {
        // Result<T, E> - return the Result to R without unwrapping
        // Uses IntoR impl which returns list(error=...) on Err
        // Note: Requires E: Display for the IntoR impl
        *ctx.is_invisible = false;
        if ok_is_sexp {
            // Still require main thread for Result<SEXP, E>
            *ctx.returns_sexp = true;
        }
        quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
    } else if ok_is_unit {
        // Result<(), E> - invisible, r_stop on Err
        // Uses Debug format so it works with any E: Debug
        *ctx.is_invisible = true;
        ctx.post_call_statements.push(quote::quote! {
            if let Err(e) = #rust_result_ident {
                ::miniextendr_api::error::r_stop(&format!("{:?}", e));
            }
        });
        quote::quote! { ::miniextendr_api::ffi::SEXP::null() }
    } else {
        // Result<T, E> - unwrap then convert
        // Uses Debug format so it works with any E: Debug
        *ctx.is_invisible = false;
        if ok_is_sexp {
            *ctx.returns_sexp = true;
        }
        ctx.post_call_statements.push(quote::quote! {
            let #rust_result_ident = match #rust_result_ident {
                Ok(v) => v,
                Err(e) => ::miniextendr_api::error::r_stop(&format!("{:?}", e)),
            };
        });
        if ok_is_sexp {
            quote::quote! { #rust_result_ident }
        } else {
            quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
        }
    }
}

/// Scalar types whose conversion between R and Rust is potentially lossy.
///
/// R represents all integers as 32-bit and all reals as 64-bit doubles,
/// so types wider than `i32` (or unsigned) may overflow or lose precision.
/// In strict mode, these types use checked conversions that panic on overflow.
pub(crate) const LOSSY_SCALARS: &[&str] = &["i64", "u64", "isize", "usize"];

/// Try to generate a strict conversion expression for a lossy return type.
///
/// Returns `Some(TokenStream)` if the type is a lossy scalar or `Vec<lossy>`,
/// otherwise `None` (falls through to standard `IntoR::into_sexp`).
pub(crate) fn strict_conversion_for_type(
    ty: &syn::Type,
    result_ident: &syn::Ident,
) -> Option<proc_macro2::TokenStream> {
    let type_name = last_segment_ident(ty)?;
    let name = type_name.to_string();

    // Check for scalar lossy types: i64, u64, isize, usize
    if LOSSY_SCALARS.contains(&name.as_str()) {
        let helper = quote::format_ident!("checked_into_sexp_{}", name);
        return Some(quote::quote! {
            ::miniextendr_api::strict::#helper(#result_ident)
        });
    }

    // Check for Option<lossy>
    if name == "Option"
        && let Some(inner) = first_type_arg_from_type(ty)
    {
        let inner_name = last_segment_ident(inner)?.to_string();
        if LOSSY_SCALARS.contains(&inner_name.as_str()) {
            let helper = quote::format_ident!("checked_option_{}_into_sexp", inner_name);
            return Some(quote::quote! {
                ::miniextendr_api::strict::#helper(#result_ident)
            });
        }
    }

    // Check for Vec<lossy> or Vec<Option<lossy>>
    if name == "Vec"
        && let Some(inner) = first_type_arg_from_type(ty)
    {
        let inner_name = last_segment_ident(inner)?.to_string();
        if LOSSY_SCALARS.contains(&inner_name.as_str()) {
            let helper = quote::format_ident!("checked_vec_{}_into_sexp", inner_name);
            return Some(quote::quote! {
                ::miniextendr_api::strict::#helper(#result_ident)
            });
        }
        // Check for Vec<Option<lossy>>
        if inner_name == "Option"
            && let Some(option_inner) = first_type_arg_from_type(inner)
        {
            let option_inner_name = last_segment_ident(option_inner)?.to_string();
            if LOSSY_SCALARS.contains(&option_inner_name.as_str()) {
                let helper =
                    quote::format_ident!("checked_vec_option_{}_into_sexp", option_inner_name);
                return Some(quote::quote! {
                    ::miniextendr_api::strict::#helper(#result_ident)
                });
            }
        }
    }

    None
}

/// Try to generate a strict conversion expression for a lossy input parameter type.
///
/// Returns `Some(TokenStream)` if the type is a lossy scalar or `Vec<lossy>`,
/// otherwise `None` (falls through to standard `TryFromSexp`).
///
/// # Parameters
/// - `ty`: The Rust parameter type to check
/// - `sexp_ident`: Identifier for the SEXP variable holding the R value
/// - `param_name`: Parameter name string (used in error messages)
pub(crate) fn strict_input_conversion_for_type(
    ty: &syn::Type,
    sexp_ident: &syn::Ident,
    param_name: &str,
) -> Option<proc_macro2::TokenStream> {
    let type_name = last_segment_ident(ty)?;
    let name = type_name.to_string();

    // Check for scalar lossy types: i64, u64, isize, usize
    if LOSSY_SCALARS.contains(&name.as_str()) {
        let helper = quote::format_ident!("checked_try_from_sexp_{}", name);
        return Some(quote::quote! {
            ::miniextendr_api::strict::#helper(#sexp_ident, #param_name)
        });
    }

    // Check for Vec<lossy>
    if name == "Vec"
        && let Some(inner) = first_type_arg_from_type(ty)
    {
        let inner_name = last_segment_ident(inner)?.to_string();
        if LOSSY_SCALARS.contains(&inner_name.as_str()) {
            let helper = quote::format_ident!("checked_vec_try_from_sexp_{}", inner_name);
            return Some(quote::quote! {
                ::miniextendr_api::strict::#helper(#sexp_ident, #param_name)
            });
        }
    }

    None
}

/// Extract the last path segment identifier from a type.
pub(crate) fn last_segment_ident(ty: &syn::Type) -> Option<&syn::Ident> {
    if let syn::Type::Path(p) = ty {
        p.path.segments.last().map(|s| &s.ident)
    } else {
        None
    }
}

/// Extract the first generic type argument from a type (e.g., `T` from `Vec<T>`).
pub(crate) fn first_type_arg_from_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(p) = ty {
        crate::first_type_argument(p.path.segments.last()?)
    } else {
        None
    }
}
