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
}

impl RustConversionBuilder {
    /// Create a new conversion builder.
    pub fn new() -> Self {
        Self {
            coerce_all: false,
            coerce_params: Vec::new(),
        }
    }

    /// Enable coercion for all parameters.
    pub fn with_coerce_all(mut self) -> Self {
        self.coerce_all = true;
        self
    }

    /// Add a parameter name that should use coercion.
    pub fn with_coerce_param(mut self, param_name: String) -> Self {
        self.coerce_params.push(param_name);
        self
    }

    /// Check if a parameter should use coercion.
    fn should_coerce(&self, param_name: &str) -> bool {
        self.coerce_all || self.coerce_params.contains(&param_name.to_string())
    }

    /// Generate conversion statement for a single parameter.
    ///
    /// Returns all statements flattened (for main-thread execution).
    pub fn build_conversion(
        &self,
        pat_type: &syn::PatType,
        sexp_ident: &syn::Ident,
    ) -> Vec<TokenStream> {
        let (owned, borrowed) = self.build_conversion_split(pat_type, sexp_ident);
        owned.into_iter().chain(borrowed).collect()
    }

    /// Generate conversion statements split for worker thread execution.
    ///
    /// For reference types like `&str`, we need to:
    /// 1. Convert SEXP to owned type (String) - runs before closure, gets moved in
    /// 2. Borrow from owned type (&str) - runs inside closure
    ///
    /// Returns: (owned_conversions, borrow_statements)
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
                    let stmt = quote_spanned! {span=>
                        let #ident = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                            .expect(#error_msg);
                    };
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
                    let owned_stmt = quote_spanned! {span=>
                        let #owned_ident: String = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                            .expect(#error_msg);
                    };
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
                    let stmt = quote_spanned! {span=>
                        let #ident: #ty = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                            .expect(#error_msg);
                    };
                    (vec![stmt], vec![])
                }
            }

            // All other types
            _ => {
                let param_name = ident.to_string();
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
                        quote_spanned! {span=>
                            let #ident: #target = {
                                let __r_val: #r_native = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                                    .expect(#error_msg_convert);
                                ::miniextendr_api::TryCoerce::<#target>::try_coerce(__r_val)
                                    .expect(#error_msg_coerce)
                            };
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
                    None => {
                        let error_msg = format!(
                            "failed to convert parameter '{}' to {}: wrong type, length, or contains NA",
                            param_name,
                            quote!(#ty)
                        );
                        quote_spanned! {span=>
                            let #ident = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                                .expect(#error_msg);
                        }
                    }
                };
                (vec![stmt], vec![])
            }
        }
    }

    /// Generate conversion statements for all parameters in a function signature.
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
