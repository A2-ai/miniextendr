//! Shared utilities for converting R SEXP parameters to Rust types.
//!
//! This module provides a builder for generating Rust conversion code from R SEXP arguments,
//! ensuring consistent behavior across standalone functions and impl methods.

use crate::miniextendr_fn::CoercionMapping;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// Builder for generating Rust conversion statements from R SEXP parameters.
///
/// Handles:
/// - Unit types `()` → identity binding
/// - `&Dots` → special wrapper with storage
/// - Slices `&[T]` → TryFromSexp
/// - `&str` → String + Borrow (for worker thread compatibility)
/// - Scalar references → DATAPTR_RO_unchecked
/// - Coercion → extract R native type + TryCoerce
/// - Default → TryFromSexp
pub struct RustConversionBuilder {
    /// Enable coercion for all parameters
    coerce_all: bool,
    /// Parameter names that should use coercion
    coerce_params: Vec<String>,
    /// Enable strict input conversion for lossy types
    strict: bool,
    /// When true, conversion failures return tagged error SEXP instead of panicking.
    /// Only used on the main-thread error_in_r path (not worker thread).
    error_in_r: bool,
    /// Parameter names with `match_arg + several_ok` — use `match_arg_vec_from_sexp` instead of `TryFromSexp`.
    match_arg_several_ok_params: Vec<String>,
}

impl RustConversionBuilder {
    /// Create a new conversion builder.
    pub fn new() -> Self {
        Self {
            coerce_all: false,
            coerce_params: Vec::new(),
            strict: false,
            error_in_r: false,
            match_arg_several_ok_params: Vec::new(),
        }
    }

    /// Enable coercion for all parameters.
    pub fn with_coerce_all(mut self) -> Self {
        self.coerce_all = true;
        self
    }

    /// Add a single parameter name that should use coercion.
    ///
    /// `param_name` is matched against the identifier in the function signature.
    /// Can be called multiple times to add several parameters.
    pub fn with_coerce_param(mut self, param_name: String) -> Self {
        self.coerce_params.push(param_name);
        self
    }

    /// Enable strict input conversion for lossy types (i64/u64/isize/usize + Vec variants).
    pub fn with_strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Enable error_in_r mode: conversion failures return tagged error SEXP instead of panicking.
    ///
    /// Only appropriate for the main-thread error_in_r path. On the worker thread path,
    /// conversions happen inside `catch_unwind`, so `.expect()` panics are caught. On the
    /// `no_error_in_r` path, panics are needed for `with_r_unwind_protect` to catch.
    pub fn with_error_in_r(mut self) -> Self {
        self.error_in_r = true;
        self
    }

    /// Mark a parameter as `match_arg + several_ok` — uses `match_arg_vec_from_sexp`
    /// instead of `TryFromSexp` for converting STRSXP → `Vec<EnumType>`.
    pub fn with_match_arg_several_ok(mut self, param_name: String) -> Self {
        self.match_arg_several_ok_params.push(param_name);
        self
    }

    /// Check if a parameter should use coercion.
    ///
    /// Returns `true` if `coerce_all` is set or `param_name` appears in the per-parameter list.
    fn should_coerce(&self, param_name: &str) -> bool {
        self.coerce_all || self.coerce_params.contains(&param_name.to_string())
    }

    /// Generate a conversion expression that either panics (`.expect()`) or returns
    /// a tagged error SEXP (`error_in_r` mode) on failure.
    ///
    /// - `try_expr`: The `Result<T, E>`-producing expression
    /// - `error_msg`: Human-readable error message for the failure
    /// - `ident`: The binding name for the converted value
    /// - `ty`: The target Rust type (for the `let` binding)
    /// - `span`: Source span for error reporting
    fn conversion_stmt(
        &self,
        try_expr: TokenStream,
        error_msg: &str,
        ident: &syn::Ident,
        ty: &syn::Type,
        span: proc_macro2::Span,
    ) -> TokenStream {
        if self.error_in_r {
            quote_spanned! {span=>
                let #ident: #ty = match #try_expr {
                    Ok(v) => v,
                    Err(e) => return ::miniextendr_api::error_value::make_rust_condition_value(
                        &format!("{}: {e}", #error_msg),
                        "conversion",
                        ::core::option::Option::None,
                        Some(__miniextendr_call),
                    ),
                };
            }
        } else {
            quote_spanned! {span=>
                let #ident: #ty = #try_expr.expect(#error_msg);
            }
        }
    }

    /// Like [`conversion_stmt`] but without a type annotation on the binding.
    fn conversion_stmt_untyped(
        &self,
        try_expr: TokenStream,
        error_msg: &str,
        ident: &syn::Ident,
        span: proc_macro2::Span,
    ) -> TokenStream {
        if self.error_in_r {
            quote_spanned! {span=>
                let #ident = match #try_expr {
                    Ok(v) => v,
                    Err(e) => return ::miniextendr_api::error_value::make_rust_condition_value(
                        &format!("{}: {e}", #error_msg),
                        "conversion",
                        ::core::option::Option::None,
                        Some(__miniextendr_call),
                    ),
                };
            }
        } else {
            quote_spanned! {span=>
                let #ident = #try_expr.expect(#error_msg);
            }
        }
    }

    /// Generate conversion statement for a single parameter.
    ///
    /// This is the non-split variant: owned conversions and borrow statements are
    /// concatenated into a single list, suitable for main-thread execution where
    /// everything runs in the same scope.
    ///
    /// - `pat_type`: the typed pattern from the function signature (e.g., `x: i32`).
    /// - `sexp_ident`: the identifier of the raw SEXP variable holding the R argument.
    ///
    /// Returns a flat list of `let` binding statements that convert `sexp_ident` into
    /// the Rust type declared in `pat_type`.
    pub fn build_conversion(
        &self,
        pat_type: &syn::PatType,
        sexp_ident: &syn::Ident,
    ) -> Vec<TokenStream> {
        let (owned, borrowed) = self.build_conversion_split(pat_type, sexp_ident);
        owned.into_iter().chain(borrowed).collect()
    }

    /// Generate conversion statements split into two phases for worker thread execution.
    ///
    /// For reference types like `&str`, we need to:
    /// 1. Convert SEXP to owned type (String) -- runs on the main thread before the
    ///    worker closure, so the owned value can be moved into the closure.
    /// 2. Borrow from the owned type (`&str`) -- runs inside the worker closure.
    ///
    /// For non-reference types (scalars, `Vec`, etc.) everything goes into the first
    /// phase and the second vec is empty.
    ///
    /// - `pat_type`: the typed pattern from the function signature (e.g., `s: &str`).
    /// - `sexp_ident`: the identifier of the raw SEXP variable holding the R argument.
    ///
    /// Returns `(owned_conversions, borrow_statements)` where each element is a list
    /// of `let` binding token streams.
    pub fn build_conversion_split(
        &self,
        pat_type: &syn::PatType,
        sexp_ident: &syn::Ident,
    ) -> (Vec<TokenStream>, Vec<TokenStream>) {
        let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            return (vec![], vec![]);
        };
        let ident = &pat_ident.ident;
        let ty = pat_type.ty.as_ref();

        match ty {
            // Unit type: ()
            // Note: We never generate `mut` on conversion bindings - the user's function
            // has its own parameter binding that will be `mut` if they specified it.
            syn::Type::Tuple(t) if t.elems.is_empty() => {
                let stmt = quote! { let #ident = (); };
                (vec![stmt], vec![])
            }

            // Reference types: &T, &mut T
            syn::Type::Reference(r) => {
                let param_name = ident.to_string();
                let is_dots = matches!(
                    r.elem.as_ref(),
                    syn::Type::Path(tp)
                        if tp.path.segments.last()
                            .map(|s| s.ident == "Dots")
                            .unwrap_or(false)
                );
                let is_slice = matches!(r.elem.as_ref(), syn::Type::Slice(_));
                let is_str = matches!(
                    r.elem.as_ref(),
                    syn::Type::Path(tp) if tp.path.is_ident("str")
                );

                // &[T] / &mut [T] with match_arg + several_ok:
                // two-phase: pre-call Vec<T>, in-call borrow
                if is_slice
                    && self
                        .match_arg_several_ok_params
                        .contains(&param_name.to_string())
                    && let Some((crate::SeveralOkContainer::BorrowedSlice, inner_ty)) =
                        crate::classify_several_ok_container(ty)
                {
                    let is_mut = r.mutability.is_some();
                    let storage_ident = quote::format_ident!("__storage_{}", ident);
                    let error_msg = format!(
                        "failed to convert parameter '{}' to &{}[{}]: invalid choice",
                        param_name,
                        if is_mut { "mut " } else { "" },
                        quote::quote!(#inner_ty)
                    );
                    let vec_ty: syn::Type = syn::parse_quote!(::std::vec::Vec<#inner_ty>);
                    let span = ty.span();
                    let try_expr = quote_spanned! {span=>
                        ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                    };
                    // Emit owned Vec<T> binding.
                    // For &mut [T] the storage binding needs `mut`.
                    let owned_stmt = if is_mut {
                        // Need `let mut storage_ident: vec_ty = try_expr.expect(...)`.
                        // Inline the mutation variant here.
                        if self.error_in_r {
                            let em = &error_msg;
                            quote_spanned! {span=>
                                let mut #storage_ident: #vec_ty = match #try_expr {
                                    Ok(v) => v,
                                    Err(e) => return ::miniextendr_api::error_value::make_rust_condition_value(
                                        &format!("{}: {e}", #em),
                                        "conversion",
                                        ::core::option::Option::None,
                                        Some(__miniextendr_call),
                                    ),
                                };
                            }
                        } else {
                            let em = &error_msg;
                            quote_spanned! {span=>
                                let mut #storage_ident: #vec_ty = #try_expr.expect(#em);
                            }
                        }
                    } else {
                        self.conversion_stmt(try_expr, &error_msg, &storage_ident, &vec_ty, span)
                    };
                    let borrow_stmt = if is_mut {
                        quote_spanned! {span=>
                            let #ident: #ty = &mut #storage_ident;
                        }
                    } else {
                        quote_spanned! {span=>
                            let #ident: #ty = &#storage_ident;
                        }
                    };
                    return (vec![owned_stmt], vec![borrow_stmt]);
                }

                if is_dots {
                    // &Dots: create wrapper with storage (main thread only - requires SEXP)
                    let storage_ident = quote::format_ident!("{}_storage", ident);
                    let stmt = quote! {
                        let #storage_ident = ::miniextendr_api::dots::Dots { inner: #sexp_ident };
                        let #ident = &#storage_ident;
                    };
                    (vec![stmt], vec![])
                } else if is_slice {
                    // &[T]: use TryFromSexp (backed by DATAPTR_RO)
                    let error_msg = format!(
                        "failed to convert parameter '{}' to slice: wrong type or length",
                        ident
                    );
                    let span = ty.span();
                    let try_expr = quote_spanned! {span=>
                        ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                    };
                    let stmt = self.conversion_stmt_untyped(try_expr, &error_msg, ident, span);
                    (vec![stmt], vec![])
                } else if is_str {
                    // &str: Convert to String, then borrow using Borrow trait.
                    // This allows the String to be moved into worker thread closures.
                    let owned_ident = quote::format_ident!("__owned_{}", ident);
                    let error_msg = format!(
                        "failed to convert parameter '{}' to string: expected character vector",
                        ident
                    );
                    let span = ty.span();
                    // Owned conversion: SEXP -> String
                    let string_ty: syn::Type = syn::parse_quote!(String);
                    let try_expr = quote_spanned! {span=>
                        ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                    };
                    let owned_stmt =
                        self.conversion_stmt(try_expr, &error_msg, &owned_ident, &string_ty, span);
                    // Borrow: String -> &str (using Borrow trait)
                    let borrow_stmt = quote_spanned! {span=>
                        let #ident: &str = ::std::borrow::Borrow::borrow(&#owned_ident);
                    };
                    (vec![owned_stmt], vec![borrow_stmt])
                } else {
                    // &T for other types: use TryFromSexp for the reference type.
                    let error_msg = format!(
                        "failed to convert parameter '{}' to {}: wrong type",
                        ident,
                        quote!(#ty)
                    );
                    let span = ty.span();
                    let try_expr = quote_spanned! {span=>
                        ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                    };
                    let stmt = self.conversion_stmt(try_expr, &error_msg, ident, ty, span);
                    (vec![stmt], vec![])
                }
            }

            // All other types
            _ => {
                let param_name = ident.to_string();

                // Strict mode: use checked input helpers for lossy types
                if self.strict
                    && let Some(strict_expr) =
                        crate::return_type_analysis::strict_input_conversion_for_type(
                            ty,
                            sexp_ident,
                            &param_name,
                        )
                {
                    let span = ty.span();
                    let stmt = quote_spanned! {span=>
                        let #ident: #ty = #strict_expr;
                    };
                    return (vec![stmt], vec![]);
                }

                // match_arg + several_ok: use match_arg_vec_from_sexp for container types
                if self
                    .match_arg_several_ok_params
                    .contains(&param_name.to_string())
                    && let Some((container, inner_ty)) = crate::classify_several_ok_container(ty)
                {
                    let span = ty.span();
                    match container {
                        crate::SeveralOkContainer::Vec => {
                            let error_msg = format!(
                                "failed to convert parameter '{}' to Vec<{}>: invalid choice",
                                param_name,
                                quote!(#inner_ty)
                            );
                            let try_expr = quote_spanned! {span=>
                                ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                            };
                            let stmt = self.conversion_stmt(try_expr, &error_msg, ident, ty, span);
                            return (vec![stmt], vec![]);
                        }
                        crate::SeveralOkContainer::BoxedSlice => {
                            let error_msg = format!(
                                "failed to convert parameter '{}' to Box<[{}]>: invalid choice",
                                param_name,
                                quote!(#inner_ty)
                            );
                            let try_expr = quote_spanned! {span=>
                                ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                                    .map(|v| v.into_boxed_slice())
                            };
                            let stmt = self.conversion_stmt(try_expr, &error_msg, ident, ty, span);
                            return (vec![stmt], vec![]);
                        }
                        crate::SeveralOkContainer::Array(n) => {
                            let error_msg = format!(
                                "failed to convert parameter '{}': invalid choice",
                                param_name,
                            );
                            let param_name_lit = &param_name;
                            let span = ty.span();
                            // First extract the Vec via match_arg_vec_from_sexp (handles
                            // match_arg validation + error reporting), then convert length-check
                            // separately via a direct panic (caught by the framework).
                            let vec_ty: syn::Type = syn::parse_quote!(::std::vec::Vec<#inner_ty>);
                            let vec_ident = quote::format_ident!("__vec_{}", ident);
                            let try_expr = quote_spanned! {span=>
                                ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                            };
                            let vec_stmt = self
                                .conversion_stmt(try_expr, &error_msg, &vec_ident, &vec_ty, span);
                            // Length check + array conversion via panic (framework catches panics)
                            let arr_stmt = quote_spanned! {span=>
                                let #ident: #ty = {
                                    if #vec_ident.len() != #n {
                                        panic!(
                                            "parameter `{}`: expected {} values for [_; {}], got {}",
                                            #param_name_lit, #n, #n, #vec_ident.len()
                                        );
                                    }
                                    <[#inner_ty; #n]>::try_from(#vec_ident)
                                        .unwrap_or_else(|_| unreachable!())
                                };
                            };
                            return (vec![vec_stmt, arr_stmt], vec![]);
                        }
                        crate::SeveralOkContainer::BorrowedSlice => {
                            let storage_ident = quote::format_ident!("__storage_{}", ident);
                            let error_msg = format!(
                                "failed to convert parameter '{}' to &[{}]: invalid choice",
                                param_name,
                                quote!(#inner_ty)
                            );
                            let vec_ty: syn::Type = syn::parse_quote!(::std::vec::Vec<#inner_ty>);
                            let try_expr = quote_spanned! {span=>
                                ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                            };
                            let owned_stmt = self.conversion_stmt(
                                try_expr,
                                &error_msg,
                                &storage_ident,
                                &vec_ty,
                                span,
                            );
                            let borrow_stmt = quote_spanned! {span=>
                                let #ident: #ty = &#storage_ident;
                            };
                            return (vec![owned_stmt], vec![borrow_stmt]);
                        }
                    }
                }

                let should_coerce = self.should_coerce(&param_name);
                let coercion_mapping = if should_coerce {
                    CoercionMapping::from_type(ty)
                } else {
                    None
                };

                let span = ty.span();
                let stmt = match coercion_mapping {
                    Some(CoercionMapping::Scalar { r_native, target }) => {
                        let error_msg_convert = format!(
                            "failed to convert parameter '{}' from R: wrong type",
                            param_name
                        );
                        let error_msg_coerce = format!(
                            "failed to coerce parameter '{}' to {}: overflow, NaN, or precision loss",
                            param_name,
                            quote!(#target)
                        );
                        if self.error_in_r {
                            quote_spanned! {span=>
                                let #ident: #target = {
                                    let __r_val: #r_native = match ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident) {
                                        Ok(v) => v,
                                        Err(e) => return ::miniextendr_api::error_value::make_rust_condition_value(
                                            &format!("{}: {e}", #error_msg_convert),
                                            "conversion",
                                            ::core::option::Option::None,
                                            Some(__miniextendr_call),
                                        ),
                                    };
                                    match ::miniextendr_api::TryCoerce::<#target>::try_coerce(__r_val) {
                                        Ok(v) => v,
                                        Err(e) => return ::miniextendr_api::error_value::make_rust_condition_value(
                                            &format!("{}: {e}", #error_msg_coerce),
                                            "conversion",
                                            ::core::option::Option::None,
                                            Some(__miniextendr_call),
                                        ),
                                    }
                                };
                            }
                        } else {
                            quote_spanned! {span=>
                                let #ident: #target = {
                                    let __r_val: #r_native = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                                        .expect(#error_msg_convert);
                                    ::miniextendr_api::TryCoerce::<#target>::try_coerce(__r_val)
                                        .expect(#error_msg_coerce)
                                };
                            }
                        }
                    }
                    Some(CoercionMapping::Vec {
                        r_native_elem,
                        target_elem,
                    }) => {
                        let error_msg_convert = format!(
                            "failed to convert parameter '{}' to vector: wrong type",
                            param_name
                        );
                        let error_msg_coerce = format!(
                            "failed to coerce parameter '{}' to Vec<{}>: element overflow, NaN, or precision loss",
                            param_name,
                            quote!(#target_elem)
                        );
                        if self.error_in_r {
                            quote_spanned! {span=>
                                let #ident: Vec<#target_elem> = {
                                    let __r_slice: &[#r_native_elem] = match ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident) {
                                        Ok(v) => v,
                                        Err(e) => return ::miniextendr_api::error_value::make_rust_condition_value(
                                            &format!("{}: {e}", #error_msg_convert),
                                            "conversion",
                                            ::core::option::Option::None,
                                            Some(__miniextendr_call),
                                        ),
                                    };
                                    match __r_slice.iter().copied()
                                        .map(::miniextendr_api::TryCoerce::<#target_elem>::try_coerce)
                                        .collect::<Result<Vec<_>, _>>()
                                    {
                                        Ok(v) => v,
                                        Err(e) => return ::miniextendr_api::error_value::make_rust_condition_value(
                                            &format!("{}: {e}", #error_msg_coerce),
                                            "conversion",
                                            ::core::option::Option::None,
                                            Some(__miniextendr_call),
                                        ),
                                    }
                                };
                            }
                        } else {
                            quote_spanned! {span=>
                                let #ident: Vec<#target_elem> = {
                                    let __r_slice: &[#r_native_elem] = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                                        .expect(#error_msg_convert);
                                    __r_slice.iter().copied()
                                        .map(::miniextendr_api::TryCoerce::<#target_elem>::try_coerce)
                                        .collect::<Result<Vec<_>, _>>()
                                        .expect(#error_msg_coerce)
                                };
                            }
                        }
                    }
                    None => {
                        let error_msg = format!(
                            "failed to convert parameter '{}' to {}: wrong type, length, or contains NA",
                            param_name,
                            quote!(#ty)
                        );
                        let try_expr = quote_spanned! {span=>
                            ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                        };
                        self.conversion_stmt(try_expr, &error_msg, ident, ty, span)
                    }
                };
                (vec![stmt], vec![])
            }
        }
    }

    /// Generate conversion statements for all parameters in a function signature.
    ///
    /// Iterates over `inputs` (the function's parameter list) paired with `sexp_idents`
    /// (the corresponding SEXP variable names), calling [`build_conversion`](Self::build_conversion)
    /// for each typed parameter. Receiver parameters (`self`) are silently skipped.
    ///
    /// Returns a flat list of all conversion statements, in parameter order.
    pub fn build_conversions(
        &self,
        inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
        sexp_idents: &[syn::Ident],
    ) -> Vec<TokenStream> {
        let mut all_statements = Vec::new();

        for (arg, sexp_ident) in inputs.iter().zip(sexp_idents.iter()) {
            if let syn::FnArg::Typed(pat_type) = arg {
                let statements = self.build_conversion(pat_type, sexp_ident);
                all_statements.extend(statements);
            }
        }

        all_statements
    }
}

impl Default for RustConversionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
