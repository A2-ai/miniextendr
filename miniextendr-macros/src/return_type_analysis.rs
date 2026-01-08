//! Return type analysis for `#[miniextendr]` functions.
//!
//! Determines:
//! 1. Whether function returns SEXP (affects thread strategy)
//! 2. Whether result should be invisible in R
//! 3. How to convert Rust return value to SEXP
//! 4. What post-call processing is needed (unwrapping Option/Result)

use crate::is_sexp_type;
use syn::spanned::Spanned;

/// Analysis result for a function's return type.
pub(crate) struct ReturnTypeAnalysis {
    /// Whether the return type contains SEXP (affects thread strategy).
    pub returns_sexp: bool,

    /// Whether the R function should return invisible (e.g., for () or Option<()>).
    pub is_invisible: bool,

    /// TokenStream converting `rust_result_ident` to SEXP.
    pub return_expression: proc_macro2::TokenStream,

    /// Statements to run after calling the Rust function (e.g., unwrap Option/Result).
    pub post_call_statements: Vec<proc_macro2::TokenStream>,
}

/// Analyze a function's return type and generate conversion code.
///
/// # Parameters
/// - `output`: The function's return type from `syn::Signature`
/// - `rust_result_ident`: Identifier for the variable holding the Rust function result
/// - `rust_ident`: Function name (for error messages)
pub(crate) fn analyze_return_type(
    output: &syn::ReturnType,
    rust_result_ident: &syn::Ident,
    rust_ident: &syn::Ident,
    return_pref: crate::miniextendr_fn::ReturnPref,
    unwrap_in_r: bool,
) -> ReturnTypeAnalysis {
    let mut returns_sexp = false;
    let mut is_invisible = false;
    let mut post_call_statements = Vec::new();

    let option_none_error_msg = quote::quote! {
        concat!(
            "miniextendr function `",
            stringify!(#rust_ident),
            "` returned None"
        )
    };

    let return_expression = match output {
        // No return type (no arrow)
        syn::ReturnType::Default => {
            is_invisible = true;
            quote::quote! { unsafe { ::miniextendr_api::ffi::R_NilValue } }
        }

        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            // -> ()
            syn::Type::Tuple(t) if t.elems.is_empty() => {
                is_invisible = true;
                quote::quote! { unsafe { ::miniextendr_api::ffi::R_NilValue } }
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
                analyze_option_type(
                    p,
                    rust_result_ident,
                    &option_none_error_msg,
                    &mut returns_sexp,
                    &mut is_invisible,
                    &mut post_call_statements,
                )
            }

            // -> Result<T, E>
            syn::Type::Path(p)
                if p.path.segments.last().map(|s| &s.ident)
                    == Some(&syn::Ident::new("Result", p.path.span())) =>
            {
                analyze_result_type(
                    p,
                    rust_result_ident,
                    &mut returns_sexp,
                    &mut is_invisible,
                    &mut post_call_statements,
                    unwrap_in_r,
                )
            }

            // -> T (any other type)
            _ => {
                is_invisible = false;
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

/// Analyze `Option<T>` return type.
fn analyze_option_type(
    type_path: &syn::TypePath,
    rust_result_ident: &syn::Ident,
    option_none_error_msg: &proc_macro2::TokenStream,
    returns_sexp: &mut bool,
    is_invisible: &mut bool,
    post_call_statements: &mut Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let seg = type_path.path.segments.last().unwrap();
    let inner_ty = crate::first_type_argument(seg);
    let is_unit_inner =
        inner_ty.is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));
    let is_sexp_inner = inner_ty.is_some_and(is_sexp_type);

    if is_unit_inner {
        // Option<()> - invisible, panic on None
        *is_invisible = true;
        post_call_statements.push(quote::quote! {
            if #rust_result_ident.is_none() {
                panic!(#option_none_error_msg);
            }
        });
        quote::quote! { unsafe { ::miniextendr_api::ffi::R_NilValue } }
    } else if is_sexp_inner {
        // Option<SEXP> - return SEXP or R_NilValue for None
        *is_invisible = false;
        *returns_sexp = true;
        quote::quote! {
            match #rust_result_ident {
                Some(v) => v,
                None => unsafe { ::miniextendr_api::ffi::R_NilValue },
            }
        }
    } else {
        // Option<T> - convert via IntoR which handles None → NA appropriately
        *is_invisible = false;
        quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
    }
}

/// Analyze Result<T, E> return type.
fn analyze_result_type(
    type_path: &syn::TypePath,
    rust_result_ident: &syn::Ident,
    returns_sexp: &mut bool,
    is_invisible: &mut bool,
    post_call_statements: &mut Vec<proc_macro2::TokenStream>,
    unwrap_in_r: bool,
) -> proc_macro2::TokenStream {
    let seg = type_path.path.segments.last().unwrap();
    let ok_ty = crate::first_type_argument(seg);
    let err_ty = crate::second_type_argument(seg);
    let ok_is_unit =
        ok_ty.is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));
    let ok_is_sexp = ok_ty.is_some_and(is_sexp_type);
    let err_is_unit =
        err_ty.is_some_and(|ty| matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()));

    // Special case: Result<T, ()> - convert to Result<T, NullOnErr> which returns NULL on Err
    if err_is_unit {
        if ok_is_unit {
            // Result<(), ()> - invisible, always returns NULL
            *is_invisible = true;
            quote::quote! { unsafe { ::miniextendr_api::ffi::R_NilValue } }
        } else {
            // Result<T, ()> - convert to Result<T, NullOnErr> and use IntoR
            // IntoR for Result<T, NullOnErr> returns NULL on Err
            *is_invisible = false;
            if ok_is_sexp {
                *returns_sexp = true;
            }
            // Convert Err(()) to Err(NullOnErr) so IntoR can return NULL
            post_call_statements.push(quote::quote! {
                let #rust_result_ident = #rust_result_ident.map_err(|()| ::miniextendr_api::into_r::NullOnErr);
            });
            // Use IntoR which returns NULL on Err(NullOnErr)
            quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
        }
    } else if unwrap_in_r {
        // Result<T, E> - return the Result to R without unwrapping
        // Uses IntoR impl which returns list(error=...) on Err
        // Note: Requires E: Display for the IntoR impl
        *is_invisible = false;
        if ok_is_sexp {
            // Still require main thread for Result<SEXP, E>
            *returns_sexp = true;
        }
        quote::quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#rust_result_ident) }
    } else if ok_is_unit {
        // Result<(), E> - invisible, panic on Err
        // Uses Debug format so it works with any E: Debug
        *is_invisible = true;
        post_call_statements.push(quote::quote! {
            if let Err(e) = #rust_result_ident {
                panic!("{:?}", e);
            }
        });
        quote::quote! { unsafe { ::miniextendr_api::ffi::R_NilValue } }
    } else {
        // Result<T, E> - unwrap then convert
        // Uses Debug format so it works with any E: Debug
        *is_invisible = false;
        if ok_is_sexp {
            *returns_sexp = true;
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
