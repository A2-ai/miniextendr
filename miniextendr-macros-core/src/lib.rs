//! Shared parser types for miniextendr proc macros and lint.
//!
//! This crate contains the `miniextendr_module!` parser and naming helper
//! functions used by both `miniextendr-macros` (proc-macro codegen) and
//! `miniextendr-lint` (build-time static analysis).

pub mod miniextendr_module;

/// Identifier for the generated `const` `R_CallMethodDef` value.
///
/// This must remain consistent between the attribute macro (which defines the symbol)
/// and the module macro (which references it).
pub fn call_method_def_ident_for(rust_ident: &syn::Ident) -> syn::Ident {
    quote::format_ident!("call_method_def_{rust_ident}")
}

/// Identifier for the generated `const &str` holding the R wrapper source code.
///
/// This must remain consistent between the attribute macro (which defines the symbol)
/// and the module macro (which references it).
pub fn r_wrapper_const_ident_for(rust_ident: &syn::Ident) -> syn::Ident {
    let rust_ident_upper = rust_ident.to_string().to_uppercase();
    quote::format_ident!("R_WRAPPER_{rust_ident_upper}")
}

/// Identifier for the generated `const [R_CallMethodDef; N]` holding match_arg
/// choices helper call defs for a function.
///
/// Every `#[miniextendr]` function generates this array (empty if no match_arg params).
/// The module macro references it for registration.
pub fn match_arg_call_defs_ident_for(rust_ident: &syn::Ident) -> syn::Ident {
    let rust_ident_upper = rust_ident.to_string().to_uppercase();
    quote::format_ident!("MATCH_ARG_CALL_DEFS_{rust_ident_upper}")
}
