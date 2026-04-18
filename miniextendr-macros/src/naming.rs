//! Naming helpers for the static symbols the `#[miniextendr]` attribute emits.
//!
//! These formatters live in one place so the attribute macro and any later
//! consumer (e.g. a registration-chasing linter) can compute the exact same
//! identifiers from a source `syn::Ident`.

/// Identifier for the generated `const R_CallMethodDef` value.
///
/// Must remain consistent between the attribute macro (which defines the
/// symbol) and any code that later looks it up. Returns
/// `call_method_def_{rust_ident}`.
pub(crate) fn call_method_def_ident_for(rust_ident: &syn::Ident) -> syn::Ident {
    quote::format_ident!("call_method_def_{rust_ident}")
}

/// Identifier for the generated `const &str` holding the R wrapper source.
///
/// Returns `R_WRAPPER_{RUST_IDENT}` (uppercased).
pub(crate) fn r_wrapper_const_ident_for(rust_ident: &syn::Ident) -> syn::Ident {
    let upper = rust_ident.to_string().to_uppercase();
    quote::format_ident!("R_WRAPPER_{upper}")
}

/// Identifier for the generated `const [R_CallMethodDef; N]` holding the
/// match_arg choices helper call defs for a function.
///
/// Every `#[miniextendr]` function emits this array (empty if no `match_arg`
/// params). Returns `MATCH_ARG_CALL_DEFS_{RUST_IDENT}` (uppercased).
#[allow(dead_code)]
pub(crate) fn match_arg_call_defs_ident_for(rust_ident: &syn::Ident) -> syn::Ident {
    let upper = rust_ident.to_string().to_uppercase();
    quote::format_ident!("MATCH_ARG_CALL_DEFS_{upper}")
}
