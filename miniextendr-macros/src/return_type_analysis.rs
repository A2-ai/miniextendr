//! Return type analysis for `#[miniextendr]` functions.
//!
//! Determines whether the result should be invisible in R.

use crate::is_sexp_type;
use syn::spanned::Spanned;

/// Mutable context threaded through `analyze_option_type` and `analyze_result_type`.
///
/// Collects analysis results that are determined while recursively inspecting
/// the inner types of `Option<T>` and `Result<T, E>`.
struct AnalysisCtx<'a> {
    /// Identifier for the variable holding the Rust function's return value.
    rust_result_ident: &'a syn::Ident,
    /// Set to `true` if the R wrapper should return invisibly (e.g., `()` or `Option<()>`).
    is_invisible: &'a mut bool,
    /// Statements to execute after calling the Rust function but before converting
    /// the result to SEXP (e.g., unwrapping `Result::Err` to an R error).
    post_call_statements: &'a mut Vec<proc_macro2::TokenStream>,
}

/// Relevant analysis result for a `#[miniextendr]` function's return type.
///
/// Captures the R wrapper's invisibility.
/// Detailed return-value codegen is handled by [`crate::c_wrapper_builder::CWrapperContext`].
pub(crate) struct ReturnTypeAnalysis {
    /// Whether the R wrapper function should return its result invisibly.
    ///
    /// Set to `true` for `()`, `Option<()>`, and `Result<(), E>` return types.
    pub is_invisible: bool,
}

/// Returns `true` if the return type is `Result<T, E>` (shallow name check).
///
/// Does not resolve type aliases. Used to decide whether `unwrap_in_r` should
/// strip the `Result` wrapper before converting to SEXP.
pub(crate) fn output_is_result(output: &syn::ReturnType) -> bool {
    match output {
        syn::ReturnType::Type(_, ty) => matches!(
            ty.as_ref(),
            syn::Type::Path(p)
                if p.path
                    .segments
                    .last()
                    .map(|s| s.ident == "Result")
                    .unwrap_or(false)
        ),
        syn::ReturnType::Default => false,
    }
}

/// Analyze a function's return type and generate conversion code.
///
/// This is the main entry point for return type analysis. It pattern-matches on the
/// return type to determine how to convert the Rust result into a SEXP for R.
///
/// # Parameters
/// - `output`: The function's return type from `syn::Signature`
/// - `rust_result_ident`: Identifier for the variable holding the Rust function result
/// - `r_name`: Resolved R function name (used in errors for `Option::None`)
/// - `unwrap_in_r`: When `true`, `Result<T, E>` is passed to R as-is via `IntoR` (list with error field)
///   rather than unwrapped in Rust (which raises an R error)
/// - `strict`: When `true`, lossy integer types (i64, u64, isize, usize) use checked
///   conversions that panic on overflow instead of silent truncation
pub(crate) fn analyze_return_type(
    output: &syn::ReturnType,
    rust_result_ident: &syn::Ident,
    r_name: &str,
    unwrap_in_r: bool,
    strict: bool,
    err_parts: &crate::c_wrapper_builder::ErrPartsMode,
) -> ReturnTypeAnalysis {
    let mut is_invisible = false;
    let mut post_call_statements = Vec::new();

    let option_none_error_msg = format!("`{r_name}()` returned no value");

    match output {
        // No return type (no arrow)
        syn::ReturnType::Default => {
            is_invisible = true;
        }

        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            // -> ()
            syn::Type::Tuple(t) if t.elems.is_empty() => {
                is_invisible = true;
            }

            // -> SEXP
            syn::Type::Path(_p) if is_sexp_type(ty.as_ref()) => {
                is_invisible = false;
            }

            // -> Option<T>
            syn::Type::Path(p)
                if p.path.segments.last().map(|s| &s.ident)
                    == Some(&syn::Ident::new("Option", p.path.span())) =>
            {
                let mut ctx = AnalysisCtx {
                    rust_result_ident,
                    is_invisible: &mut is_invisible,
                    post_call_statements: &mut post_call_statements,
                };
                analyze_option_type(p, &mut ctx, &option_none_error_msg, strict);
            }

            // -> Result<T, E>
            syn::Type::Path(p)
                if p.path.segments.last().map(|s| &s.ident)
                    == Some(&syn::Ident::new("Result", p.path.span())) =>
            {
                let mut ctx = AnalysisCtx {
                    rust_result_ident,
                    is_invisible: &mut is_invisible,
                    post_call_statements: &mut post_call_statements,
                };
                analyze_result_type(p, &mut ctx, unwrap_in_r, err_parts);
            }

            // -> T (any other type)
            _ => {
                is_invisible = false;
            }
        },
    };

    ReturnTypeAnalysis { is_invisible }
}

/// Analyze `Option<T>` return type and generate conversion code.
///
/// Handles three cases:
/// - `Option<()>`: invisible, `None` returns a tagged condition SEXP
/// - `Option<SEXP>`: returns the SEXP or `R_NilValue` for `None`
/// - `Option<T>`: delegates to `IntoR` (which maps `None` to `NA` for supported types)
fn analyze_option_type(
    type_path: &syn::TypePath,
    ctx: &mut AnalysisCtx,
    option_none_error_msg: &str,
    strict: bool,
) -> proc_macro2::TokenStream {
    let rust_result_ident = ctx.rust_result_ident;
    let seg = type_path.path.segments.last().unwrap();
    let inner_ty = crate::first_type_argument(seg);
    let is_unit_inner =
        inner_ty.is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));
    let is_sexp_inner = inner_ty.is_some_and(is_sexp_type);

    if is_unit_inner {
        // Option<()> - invisible; tagged condition value on None.
        *ctx.is_invisible = true;
        quote::quote! {
            match #rust_result_ident {
                ::core::option::Option::Some(()) => ::miniextendr_api::SEXP::nil(),
                // SAFETY: runs inside the wrapper's with_r_unwind_protect closure on the R main thread.
                ::core::option::Option::None => unsafe { ::miniextendr_api::error_value::make_rust_condition_value(
                    #option_none_error_msg, ::miniextendr_api::error_value::kind::NONE_ERR, ::core::option::Option::None, ::core::option::Option::Some(__miniextendr_call),
                ) },
            }
        }
    } else if is_sexp_inner {
        // Option<SEXP> - return SEXP or R_NilValue for None
        *ctx.is_invisible = false;
        quote::quote! {
            match #rust_result_ident {
                ::core::option::Option::Some(v) => v,
                ::core::option::Option::None => ::miniextendr_api::SEXP::nil(),
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
/// - `Result<(), E>`: invisible; `Err` returns a tagged condition SEXP
/// - `Result<SEXP, E>`: returns the SEXP directly on `Ok`, tagged condition on `Err`
/// - `Result<T, E>` with `unwrap_in_r`: passes the full `Result` to R as a list
/// - `Result<T, E>` default: tagged condition SEXP on `Err`, `IntoR` on `Ok`
fn analyze_result_type(
    type_path: &syn::TypePath,
    ctx: &mut AnalysisCtx,
    unwrap_in_r: bool,
    err_parts: &crate::c_wrapper_builder::ErrPartsMode,
) -> proc_macro2::TokenStream {
    let rust_result_ident = ctx.rust_result_ident;
    let err_parts = err_parts.expr();
    let seg = type_path.path.segments.last().unwrap();
    let ok_ty = crate::first_type_argument(seg);
    let err_ty = crate::second_type_argument(seg);
    let ok_is_unit =
        ok_ty.is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));
    let ok_is_sexp = ok_ty.is_some_and(is_sexp_type);
    let err_is_unit =
        err_ty.is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));

    // Special case: Result<T, ()> - convert to Result<T, NullOnErr> which returns NULL on Err.
    // Unit error is a deliberate sentinel, not a Rust failure.
    if err_is_unit {
        if ok_is_unit {
            // Result<(), ()> - invisible, always returns NULL
            *ctx.is_invisible = true;
            quote::quote! { ::miniextendr_api::SEXP::nil() }
        } else {
            // Result<T, ()> - convert to Result<T, NullOnErr> and use IntoR
            // IntoR for Result<T, NullOnErr> returns NULL on Err
            *ctx.is_invisible = false;
            // Convert Err(()) to Err(NullOnErr) so IntoR can return NULL
            ctx.post_call_statements.push(quote::quote! {
                let #rust_result_ident = #rust_result_ident.map_err(|()| ::miniextendr_api::into_r::NullOnErr);
            });
            // Use IntoR which returns NULL on Err(NullOnErr)
            quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
        }
    } else if unwrap_in_r {
        // Result<T, E> - return the Result to R without unwrapping (takes priority
        // over the default tagged-condition path).
        // Uses IntoR impl which returns list(error=...) on Err.
        // Note: Requires E: Display for the IntoR impl.
        *ctx.is_invisible = false;
        quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
    } else if ok_is_unit {
        // Result<(), E> - invisible, tagged condition SEXP on Err
        *ctx.is_invisible = true;
        quote::quote! {
            match #rust_result_ident {
                ::core::result::Result::Ok(()) => ::miniextendr_api::SEXP::nil(),
                // SAFETY: runs inside the wrapper's with_r_unwind_protect closure on the R main thread.
                ::core::result::Result::Err(e) => unsafe { ::miniextendr_api::error_value::result_err_condition_value(
                            #err_parts, ::core::option::Option::Some(__miniextendr_call),
                        ) },
            }
        }
    } else if ok_is_sexp {
        // Result<SEXP, E> - return SEXP or tagged condition SEXP on Err
        *ctx.is_invisible = false;
        quote::quote! {
            match #rust_result_ident {
                ::core::result::Result::Ok(v) => v,
                // SAFETY: runs inside the wrapper's with_r_unwind_protect closure on the R main thread.
                ::core::result::Result::Err(e) => unsafe { ::miniextendr_api::error_value::result_err_condition_value(
                            #err_parts, ::core::option::Option::Some(__miniextendr_call),
                        ) },
            }
        }
    } else {
        // Result<T, E> - convert Ok to SEXP via IntoR, tagged condition SEXP on Err
        *ctx.is_invisible = false;
        quote::quote! {
            match #rust_result_ident {
                ::core::result::Result::Ok(v) => ::miniextendr_api::into_r::IntoR::into_sexp(v),
                // SAFETY: runs inside the wrapper's with_r_unwind_protect closure on the R main thread.
                ::core::result::Result::Err(e) => unsafe { ::miniextendr_api::error_value::result_err_condition_value(
                            #err_parts, ::core::option::Option::Some(__miniextendr_call),
                        ) },
            }
        }
    }
}

/// Scalar types whose conversion between R and Rust is potentially lossy.
///
/// R represents all integers as 32-bit and all reals as 64-bit doubles,
/// so types wider than `i32` (or unsigned) may overflow or lose precision.
/// In strict mode, these types use checked conversions that panic on overflow.
pub(crate) const LOSSY_SCALARS: &[&str] = &["i64", "u64", "isize", "usize"];

/// The lossy scalar `T` when `ty` is `Vec<T>` with `T` in [`LOSSY_SCALARS`].
fn lossy_vec_element(ty: &syn::Type) -> Option<String> {
    if last_segment_ident(ty)? != "Vec" {
        return None;
    }
    let elem = last_segment_ident(first_type_arg_from_type(ty)?)?.to_string();
    LOSSY_SCALARS.contains(&elem.as_str()).then_some(elem)
}

/// A contiguous return shape whose conversion can borrow its elements.
/// Vec has existing named helpers; these are the other owned/borrowed shapes.
fn lossy_slice_element(ty: &syn::Type) -> Option<&syn::Type> {
    let elem = match ty {
        syn::Type::Reference(reference) => return lossy_slice_element(&reference.elem),
        syn::Type::Slice(slice) => slice.elem.as_ref(),
        syn::Type::Array(array) => array.elem.as_ref(),
        syn::Type::Paren(paren) => return lossy_slice_element(&paren.elem),
        syn::Type::Group(group) => return lossy_slice_element(&group.elem),
        _ if last_segment_ident(ty).is_some_and(|name| name == "Box") => {
            return lossy_slice_element(first_type_arg_from_type(ty)?);
        }
        _ => return None,
    };
    let name = last_segment_ident(elem)?.to_string();
    LOSSY_SCALARS.contains(&name.as_str()).then_some(elem)
}

fn strict_slice_conversion(
    ty: &syn::Type,
    result: &syn::Ident,
) -> Option<proc_macro2::TokenStream> {
    let elem = lossy_slice_element(ty)?;
    let container = crate::type_inspect::type_display(ty);
    Some(quote::quote! {
        ::miniextendr_api::strict::checked_slice_into_sexp::<#elem>(
            #container, ::core::convert::AsRef::as_ref(&#result)
        )
    })
}

/// Try to generate a strict conversion expression for a lossy return type.
///
/// Returns `Some(TokenStream)` if the type is a lossy scalar, `Option<lossy>`,
/// `Vec<lossy>`, `Vec<Option<lossy>>`, or one of the list-of-vectors shapes
/// `Option<Vec<lossy>>`, `Vec<Vec<lossy>>`, `Vec<Option<Vec<lossy>>>`.
/// Slices, boxed slices and arrays use the same checks, including optional
/// containers and rows. Otherwise `None` (falls through to standard `IntoR::into_sexp`).
pub(crate) fn strict_conversion_for_type(
    ty: &syn::Type,
    result_ident: &syn::Ident,
) -> Option<proc_macro2::TokenStream> {
    if let Some(conversion) = strict_slice_conversion(ty, result_ident) {
        return Some(conversion);
    }
    let type_name = last_segment_ident(ty)?;
    let name = type_name.to_string();

    // Check for scalar lossy types: i64, u64, isize, usize
    if LOSSY_SCALARS.contains(&name.as_str()) {
        let helper = quote::format_ident!("checked_into_sexp_{}", name);
        return Some(quote::quote! {
            ::miniextendr_api::strict::#helper(#result_ident)
        });
    }

    // Check for Option<lossy> or Option<Vec<lossy>>
    if name == "Option"
        && let Some(inner) = first_type_arg_from_type(ty)
    {
        let value = syn::Ident::new("__mx_strict_slice", proc_macro2::Span::mixed_site());
        if let Some(conversion) = strict_slice_conversion(inner, &value) {
            return Some(quote::quote! {
                match #result_ident {
                    ::core::option::Option::Some(#value) => #conversion,
                    ::core::option::Option::None => ::miniextendr_api::SEXP::nil(),
                }
            });
        }
        // Option<Vec<lossy>>: `Some` is the checked vector, `None` is NULL
        // (what the lax `Option<Vec<T>>` impl gives `None`).
        if let Some(elem) = lossy_vec_element(inner) {
            let helper = quote::format_ident!("checked_vec_{}_into_sexp", elem);
            return Some(quote::quote! {
                match #result_ident {
                    ::core::option::Option::Some(__mx_strict_vec) => {
                        ::miniextendr_api::strict::#helper(__mx_strict_vec)
                    }
                    ::core::option::Option::None => ::miniextendr_api::SEXP::nil(),
                }
            });
        }
        let inner_name = last_segment_ident(inner)?.to_string();
        if LOSSY_SCALARS.contains(&inner_name.as_str()) {
            let helper = quote::format_ident!("checked_option_{}_into_sexp", inner_name);
            return Some(quote::quote! {
                ::miniextendr_api::strict::#helper(#result_ident)
            });
        }
    }

    // Check for Vec<lossy>, Vec<Option<lossy>>, Vec<Vec<lossy>>, Vec<Option<Vec<lossy>>>
    if name == "Vec"
        && let Some(inner) = first_type_arg_from_type(ty)
    {
        let optional = last_segment_ident(inner).is_some_and(|name| name == "Option");
        let row = if optional {
            first_type_arg_from_type(inner)?
        } else {
            inner
        };
        if let Some(elem) = lossy_slice_element(row) {
            let rows = if optional {
                quote::quote! { ::core::iter::IntoIterator::into_iter(#result_ident) }
            } else {
                quote::quote! {
                    ::core::iter::Iterator::map(
                        ::core::iter::IntoIterator::into_iter(#result_ident),
                        ::core::option::Option::Some,
                    )
                }
            };
            let container = crate::type_inspect::type_display(ty);
            return Some(quote::quote! {
                ::miniextendr_api::strict::checked_rows_into_sexp::<#elem, _>(
                    #container, #rows
                )
            });
        }
        if let Some(elem) = lossy_vec_element(inner) {
            let helper = quote::format_ident!("checked_vec_vec_{}_into_sexp", elem);
            return Some(quote::quote! {
                ::miniextendr_api::strict::#helper(#result_ident)
            });
        }
        let inner_name = last_segment_ident(inner)?.to_string();
        if LOSSY_SCALARS.contains(&inner_name.as_str()) {
            let helper = quote::format_ident!("checked_vec_{}_into_sexp", inner_name);
            return Some(quote::quote! {
                ::miniextendr_api::strict::#helper(#result_ident)
            });
        }
        // Check for Vec<Option<lossy>> and Vec<Option<Vec<lossy>>>
        if inner_name == "Option"
            && let Some(option_inner) = first_type_arg_from_type(inner)
        {
            if let Some(elem) = lossy_vec_element(option_inner) {
                let helper = quote::format_ident!("checked_vec_option_vec_{}_into_sexp", elem);
                return Some(quote::quote! {
                    ::miniextendr_api::strict::#helper(#result_ident)
                });
            }
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
/// Returns `Some(TokenStream)` if the type is a lossy scalar, `Vec<lossy>`,
/// or `Vec<Option<lossy>>`, otherwise `None` (falls through to standard
/// `TryFromSexp`). The expression is a `Result<_, SexpError>`: the caller
/// binds it like any other conversion, so a rejected input is the argument
/// error (`kind = "conversion"`, `e$param`, `e$rust_type`), not a panic.
///
/// # Parameters
/// - `ty`: The Rust parameter type to check
/// - `sexp_ident`: Identifier for the SEXP variable holding the R value
pub(crate) fn strict_input_conversion_for_type(
    ty: &syn::Type,
    sexp_ident: &syn::Ident,
) -> Option<proc_macro2::TokenStream> {
    let type_name = last_segment_ident(ty)?;
    let name = type_name.to_string();

    // Check for scalar lossy types: i64, u64, isize, usize
    if LOSSY_SCALARS.contains(&name.as_str()) {
        let helper = quote::format_ident!("checked_try_from_sexp_{}", name);
        return Some(quote::quote! {
            ::miniextendr_api::strict::#helper(#sexp_ident)
        });
    }

    // Check for Vec<lossy> or Vec<Option<lossy>>
    if name == "Vec"
        && let Some(inner) = first_type_arg_from_type(ty)
    {
        let inner_name = last_segment_ident(inner)?.to_string();
        if LOSSY_SCALARS.contains(&inner_name.as_str()) {
            let helper = quote::format_ident!("checked_vec_try_from_sexp_{}", inner_name);
            return Some(quote::quote! {
                ::miniextendr_api::strict::#helper(#sexp_ident)
            });
        }
        // Check for Vec<Option<lossy>> — must apply the same input-SEXP-type
        // gate as Vec<lossy> (reject LGLSXP/RAWSXP), not fall through to the
        // coercing TryFromSexp path. NA elements still become `None`.
        if inner_name == "Option"
            && let Some(option_inner) = first_type_arg_from_type(inner)
        {
            let option_inner_name = last_segment_ident(option_inner)?.to_string();
            if LOSSY_SCALARS.contains(&option_inner_name.as_str()) {
                let helper =
                    quote::format_ident!("checked_vec_option_try_from_sexp_{}", option_inner_name);
                return Some(quote::quote! {
                    ::miniextendr_api::strict::#helper(#sexp_ident)
                });
            }
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

#[cfg(test)]
mod tests;
