//! Shared utilities for converting R SEXP parameters to Rust types.
//!
//! This module provides a builder for generating Rust conversion code from R SEXP arguments,
//! ensuring consistent behavior across standalone functions and impl methods.

use crate::miniextendr_fn::CoercionMapping;
use proc_macro2::TokenStream;
use quote::quote;

/// Builder for generating Rust conversion statements from R SEXP parameters.
///
/// Handles:
/// - Unit types `()` → identity binding
/// - `&Dots` → special wrapper with storage
/// - Slices `&[T]` → TryFromSexp
/// - `&str` → String storage + borrow
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
    /// # Arguments
    /// * `pat_type` - The parameter pattern and type
    /// * `sexp_ident` - The SEXP identifier to convert from (e.g., `arg_0`)
    ///
    /// # Returns
    /// Vector of TokenStreams representing the conversion statements
    pub fn build_conversion(
        &self,
        pat_type: &syn::PatType,
        sexp_ident: &syn::Ident,
    ) -> Vec<TokenStream> {
        let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            return Vec::new();
        };
        let ident = &pat_ident.ident;
        let ty = pat_type.ty.as_ref();

        let mut statements = Vec::new();

        match ty {
            // Unit type: ()
            syn::Type::Tuple(t) if t.elems.is_empty() => {
                if pat_ident.mutability.is_some() {
                    statements.push(quote! { let mut #ident = (); });
                } else {
                    statements.push(quote! { let #ident = (); });
                }
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
                    // &Dots: create wrapper with storage
                    let storage_ident = quote::format_ident!("{}_storage", ident);
                    statements.push(quote! {
                        let #storage_ident = ::miniextendr_api::dots::Dots { inner: #sexp_ident };
                        let #ident = &#storage_ident;
                    });
                } else if is_slice {
                    // &[T]: use TryFromSexp (backed by DATAPTR_RO)
                    statements.push(quote! {
                        let #ident = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident).unwrap();
                    });
                } else if is_str {
                    // &str: decode to String first, then borrow
                    // This avoids returning a 'static borrow into R memory
                    let storage_ident = quote::format_ident!("__miniextendr_{}_string", ident);
                    let mutability = if pat_ident.mutability.is_some() {
                        quote!(mut)
                    } else {
                        quote!()
                    };
                    statements.push(quote! {
                        let #storage_ident: String = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident).unwrap();
                    });
                    statements.push(quote! {
                        let #mutability #ident: &str = #storage_ident.as_str();
                    });
                } else if pat_ident.mutability.is_some() {
                    // &mut T: mutable scalar reference
                    statements.push(quote! {
                        let mut #ident = unsafe { *::miniextendr_api::ffi::DATAPTR_unchecked(#sexp_ident).cast() };
                    });
                } else {
                    // &T: immutable scalar reference
                    statements.push(quote! {
                        let #ident = unsafe { *::miniextendr_api::ffi::DATAPTR_RO_unchecked(#sexp_ident).cast() };
                    });
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

                match coercion_mapping {
                    Some(CoercionMapping::Scalar { r_native, target }) => {
                        // Scalar coercion: extract R native, coerce to target
                        let mutability = if pat_ident.mutability.is_some() {
                            quote!(mut)
                        } else {
                            quote!()
                        };
                        statements.push(quote! {
                            let #mutability #ident: #target = {
                                let __r_val: #r_native = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident).unwrap();
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
                            quote!(mut)
                        } else {
                            quote!()
                        };
                        statements.push(quote! {
                            let #mutability #ident: Vec<#target_elem> = {
                                let __r_slice: &[#r_native_elem] = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident).unwrap();
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
                            statements.push(quote! {
                                let mut #ident = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident).unwrap();
                            });
                        } else {
                            statements.push(quote! {
                                let #ident = ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident).unwrap();
                            });
                        }
                    }
                }
            }
        }

        statements
    }

    /// Generate conversion statements for all parameters in a function signature.
    ///
    /// # Arguments
    /// * `inputs` - Function parameters
    /// * `sexp_idents` - SEXP identifiers for each parameter (same length as inputs)
    ///
    /// # Returns
    /// Vector of TokenStreams representing all conversion statements
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
mod tests {
    use super::*;

    fn parse_param(s: &str) -> syn::FnArg {
        let sig: syn::Signature = syn::parse_str(&format!("fn test({})", s)).unwrap();
        sig.inputs.into_iter().next().unwrap()
    }

    #[test]
    fn test_unit_type() {
        let builder = RustConversionBuilder::new();
        let param = parse_param("_unused: ()");
        if let syn::FnArg::Typed(pat_type) = param {
            let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
            let stmts = builder.build_conversion(&pat_type, &sexp_ident);
            assert_eq!(stmts.len(), 1);
            assert!(stmts[0].to_string().contains("let"));
        }
    }

    #[test]
    fn test_basic_conversion() {
        let builder = RustConversionBuilder::new();
        let param = parse_param("x: i32");
        if let syn::FnArg::Typed(pat_type) = param {
            let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
            let stmts = builder.build_conversion(&pat_type, &sexp_ident);
            assert_eq!(stmts.len(), 1);
            assert!(stmts[0].to_string().contains("TryFromSexp"));
        }
    }

    #[test]
    fn test_slice_conversion() {
        let builder = RustConversionBuilder::new();
        let param = parse_param("x: &[i32]");
        if let syn::FnArg::Typed(pat_type) = param {
            let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
            let stmts = builder.build_conversion(&pat_type, &sexp_ident);
            assert_eq!(stmts.len(), 1);
            assert!(stmts[0].to_string().contains("TryFromSexp"));
        }
    }

    #[test]
    fn test_str_conversion() {
        let builder = RustConversionBuilder::new();
        let param = parse_param("s: &str");
        if let syn::FnArg::Typed(pat_type) = param {
            let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
            let stmts = builder.build_conversion(&pat_type, &sexp_ident);
            assert_eq!(stmts.len(), 2); // String storage + borrow
            assert!(stmts[0].to_string().contains("String"));
            assert!(stmts[1].to_string().contains("as_str"));
        }
    }

    #[test]
    fn test_coercion() {
        let builder = RustConversionBuilder::new().with_coerce_param("x".to_string());
        let param = parse_param("x: u16");
        if let syn::FnArg::Typed(pat_type) = param {
            let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
            let stmts = builder.build_conversion(&pat_type, &sexp_ident);
            assert_eq!(stmts.len(), 1);
            assert!(stmts[0].to_string().contains("TryCoerce"));
            assert!(stmts[0].to_string().contains("u16"));
        }
    }
}
