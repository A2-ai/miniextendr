//! Write-time placeholder & symbol-name formatting for the match_arg pipeline.
//!
//! All four shapes (`choices_placeholder`, `param_doc_placeholder`,
//! `choices_helper_c_name`, `choices_helper_def_ident`) share the same
//! `{c_ident_without_prefix}_{r_param}` stem so the cdylib's write-time pass
//! can correlate them. Keep them together so the shape can't drift.
//!
//! The `c_ident_without_prefix` input has `C_` already stripped (or is a stem
//! already, e.g. `MyType__method`). Callers that have a full `c_ident` should
//! pass `c_ident.trim_start_matches("C_")`.
//!
//! All four helpers call `c_stem(...)` internally so callers may also pass a
//! full `c_ident` (e.g. `"C_my_fn"`) — the `C_` prefix is normalized away.

fn c_stem(c_ident: &str) -> &str {
    c_ident.trim_start_matches("C_")
}

/// R-side placeholder that the cdylib resolves to a `c("a", "b", ...)` literal
/// at write time. Substituted by `MX_MATCH_ARG_CHOICES` entries.
pub(crate) fn choices_placeholder(c_ident: &str, r_param: &str) -> String {
    format!(".__MX_MATCH_ARG_CHOICES_{}_{}__", c_stem(c_ident), r_param)
}

/// R-side placeholder for the `@param` doc line, substituted by
/// `MX_MATCH_ARG_PARAM_DOCS` entries at write time. See #210.
pub(crate) fn param_doc_placeholder(c_ident: &str, r_param: &str) -> String {
    format!(
        ".__MX_MATCH_ARG_PARAM_DOC_{}_{}__",
        c_stem(c_ident),
        r_param
    )
}

/// C symbol name for the helper fn that returns the enum's choices SEXP,
/// called from the R wrapper's match.arg prelude.
pub(crate) fn choices_helper_c_name(c_ident: &str, r_param: &str) -> String {
    format!("C_{}__match_arg_choices__{}", c_stem(c_ident), r_param)
}

/// Rust ident holding the `R_CallMethodDef` for the match_arg choices helper.
pub(crate) fn choices_helper_def_ident(c_ident: &str, r_param: &str) -> syn::Ident {
    quote::format_ident!(
        "call_method_def_{}",
        choices_helper_c_name(c_ident, r_param)
    )
}
