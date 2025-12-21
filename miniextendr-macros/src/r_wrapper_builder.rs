//! Shared utilities for building R wrapper code.
//!
//! This module provides builders for constructing R function signatures and call arguments
//! consistently across both standalone functions and impl methods.

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned;

/// Normalizes Rust argument identifiers for R.
///
/// - Leading `_` → prepends "unused"
/// - Leading `__` → prepends "private"
/// - Otherwise → unchanged
///
/// # Examples
/// - `_x` → `unused_x`
/// - `__field` → `private__field`
/// - `value` → `value`
pub fn normalize_r_arg_ident(rust_ident: &syn::Ident) -> syn::Ident {
    let mut arg_name = rust_ident.to_string();
    if arg_name.starts_with("__") {
        arg_name.insert_str(0, "private");
    } else if arg_name.starts_with('_') {
        arg_name.insert_str(0, "unused");
    }
    syn::Ident::new(&arg_name, rust_ident.span())
}

/// Builder for R function formal parameters and call arguments.
///
/// Handles:
/// - Underscore normalization (`_x` → `unused_x`)
/// - Unit type defaults (`()` → `= NULL`)
/// - Dots (`...`) with optional naming
/// - Consistent formatting across function and method wrappers
pub struct RArgumentBuilder<'a> {
    inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    /// If true, last parameter is treated as dots (`...`)
    has_dots: bool,
    /// Optional named binding for dots (e.g., `args = ...`)
    named_dots: Option<String>,
    /// If true, skip the first parameter (used for `self` in method wrappers)
    skip_first: bool,
}

impl<'a> RArgumentBuilder<'a> {
    /// Create a new builder for the given function inputs.
    pub fn new(inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> Self {
        Self {
            inputs,
            has_dots: false,
            named_dots: None,
            skip_first: false,
        }
    }

    /// Mark the last parameter as dots (`...`).
    pub fn with_dots(mut self, named_dots: Option<String>) -> Self {
        self.has_dots = true;
        self.named_dots = named_dots.map(|s| normalize_r_arg_ident(&syn::Ident::new(&s, proc_macro2::Span::call_site())).to_string());
        self
    }

    /// Skip the first parameter (for instance methods with `self`).
    pub fn skip_first(mut self) -> Self {
        self.skip_first = true;
        self
    }

    /// Build R formal parameters string (for function signature).
    ///
    /// # Returns
    /// Comma-separated parameter list, e.g., `"x, y = NULL, ..."`
    pub fn build_formals(&self) -> String {
        self.build_formals_tokens()
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Build R formal parameters as TokenStream (for macro generation).
    pub fn build_formals_tokens(&self) -> Vec<TokenStream> {
        let mut formals = Vec::new();
        let last_idx = self.inputs.len().saturating_sub(1);

        for (idx, input) in self.inputs.iter().enumerate() {
            // Skip first if requested (for self in methods)
            if self.skip_first && idx == 0 {
                continue;
            }

            let syn::FnArg::Typed(pat_type) = input else {
                continue;
            };

            // Handle dots special case
            if self.has_dots && idx == last_idx {
                if let Some(ref named) = self.named_dots {
                    let named_ident = syn::Ident::new(named, pat_type.span());
                    formals.push(syn::parse_quote!(#named_ident = ...));
                } else {
                    formals.push(syn::parse_quote!(...));
                }
                continue;
            }

            // Extract and normalize argument name
            let arg_ident = match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => normalize_r_arg_ident(&pat_ident.ident),
                _ => continue,
            };

            // Add default for unit types
            match pat_type.ty.as_ref() {
                syn::Type::Tuple(t) if t.elems.is_empty() => {
                    formals.push(syn::parse_quote!(#arg_ident = NULL));
                }
                _ => {
                    formals.push(arg_ident.into_token_stream());
                }
            }
        }

        formals
    }

    /// Build R call arguments string (for `.Call()` invocation).
    ///
    /// # Returns
    /// Comma-separated argument list, e.g., `"x, y, list(...)"`
    pub fn build_call_args(&self) -> String {
        self.build_call_args_vec().join(", ")
    }

    /// Build R call arguments as Vec<String>.
    pub fn build_call_args_vec(&self) -> Vec<String> {
        let mut call_args = Vec::new();
        let last_idx = self.inputs.len().saturating_sub(1);

        for (idx, input) in self.inputs.iter().enumerate() {
            // Skip first if requested (for self in methods)
            if self.skip_first && idx == 0 {
                continue;
            }

            let syn::FnArg::Typed(pat_type) = input else {
                continue;
            };

            // Handle dots special case
            if self.has_dots && idx == last_idx {
                if let Some(ref named) = self.named_dots {
                    call_args.push(format!("list({})", named));
                } else {
                    call_args.push("list(...)".to_string());
                }
                continue;
            }

            // Extract and normalize argument name
            let arg_ident = match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => normalize_r_arg_ident(&pat_ident.ident),
                _ => continue,
            };

            call_args.push(arg_ident.to_string());
        }

        call_args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_inputs(s: &str) -> syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma> {
        let signature: syn::Signature = syn::parse_str(&format!("fn test({})", s)).unwrap();
        signature.inputs
    }

    #[test]
    fn test_normalize_arg_ident() {
        let ident = syn::Ident::new("_x", proc_macro2::Span::call_site());
        assert_eq!(normalize_r_arg_ident(&ident).to_string(), "unused_x");

        let ident = syn::Ident::new("__private", proc_macro2::Span::call_site());
        assert_eq!(normalize_r_arg_ident(&ident).to_string(), "private__private");

        let ident = syn::Ident::new("value", proc_macro2::Span::call_site());
        assert_eq!(normalize_r_arg_ident(&ident).to_string(), "value");
    }

    #[test]
    fn test_basic_formals() {
        let inputs = parse_inputs("x: i32, y: f64");
        let builder = RArgumentBuilder::new(&inputs);
        assert_eq!(builder.build_formals(), "x, y");
    }

    #[test]
    fn test_unit_type_default() {
        let inputs = parse_inputs("x: i32, _unused: ()");
        let builder = RArgumentBuilder::new(&inputs);
        assert_eq!(builder.build_formals(), "x, unused_unused = NULL");
    }

    #[test]
    fn test_dots() {
        let inputs = parse_inputs("x: i32, _dots: &Dots");
        let builder = RArgumentBuilder::new(&inputs).with_dots(None);
        assert_eq!(builder.build_formals(), "x, ...");
        assert_eq!(builder.build_call_args(), "x, list(...)");
    }

    #[test]
    fn test_named_dots() {
        let inputs = parse_inputs("x: i32, _dots: &Dots");
        let builder = RArgumentBuilder::new(&inputs).with_dots(Some("args".to_string()));
        assert_eq!(builder.build_formals(), "x, args = ...");
        assert_eq!(builder.build_call_args(), "x, list(args)");
    }

    #[test]
    fn test_skip_first() {
        let inputs = parse_inputs("&self, x: i32, y: f64");
        let builder = RArgumentBuilder::new(&inputs).skip_first();
        assert_eq!(builder.build_formals(), "x, y");
        assert_eq!(builder.build_call_args(), "x, y");
    }

    #[test]
    fn test_underscore_normalization() {
        let inputs = parse_inputs("_x: i32, __private: String");
        let builder = RArgumentBuilder::new(&inputs);
        assert_eq!(builder.build_formals(), "unused_x, private__private");
    }
}
